//! Exercise real signed human messages through both production listener boundaries.
use super::*;

#[tokio::test]
async fn strict_allowlist_admits_private_channel_from_signed_metadata() {
    for listener in [ListenerBoundary::Normal, ListenerBoundary::Setup] {
        let channel_id = Uuid::new_v4();
        let metadata = serde_json::json!([{
            "tags": [["d", channel_id.to_string()], ["name", "private stream"], ["t", "private"]]
        }]);
        let (resolver, _, metadata_server) = lazy_resolver_with_response(metadata).await;
        let sender = nostr::Keys::generate();
        let author = sender.public_key().to_hex();
        let agent = nostr::Keys::generate().public_key().to_hex();
        let relay = nostr::Keys::generate().public_key().to_hex();
        let (rest, nip11_server) = nip11_server(serde_json::json!({ "self": relay })).await;
        let mut gate = InboundAuthorGate::connect(&rest, &agent, "private stream", true).await;
        let owner = OwnerCache::new(Some(nostr::Keys::generate().public_key().to_hex()));
        let event = relay::BuzzEvent {
            connection_generation: 0,
            channel_id,
            event: nostr::EventBuilder::new(
                nostr::Kind::Custom(KIND_STREAM_MESSAGE as u16),
                "Private stream mention",
            )
            .tags([
                nostr::Tag::parse(["h", &channel_id.to_string()]).unwrap(),
                nostr::Tag::parse(["p", &agent]).unwrap(),
            ])
            .sign_with_keys(&sender)
            .unwrap(),
        };
        event.event.verify().unwrap();
        let allowlist = HashSet::from([author.clone()]);
        let decision = gate
            .evaluate_listener_event(
                &event,
                &RespondTo::Allowlist,
                &allowlist,
                &owner,
                &resolver,
                &rest,
            )
            .await;
        assert_eq!(decision.channel_trust, "channel");
        assert!(
            decision.allowed,
            "{} rejected allowlisted private stream author",
            listener.name()
        );
        let authorized = match listener {
            ListenerBoundary::Normal => {
                authorize_normal_listener_event(
                    &mut gate,
                    event.clone(),
                    &RespondTo::Allowlist,
                    &allowlist,
                    &owner,
                    &resolver,
                    &rest,
                )
                .await
            }
            ListenerBoundary::Setup => {
                setup_mode::authorize_setup_listener_event(
                    &mut gate,
                    event.clone(),
                    &RespondTo::Allowlist,
                    &allowlist,
                    &owner,
                    &resolver,
                    &rest,
                )
                .await
            }
        };
        assert_eq!(authorized.unwrap().into_parts().1, author);
        assert!(
            gate.authorize_listener_event(
                event,
                &RespondTo::Allowlist,
                &HashSet::new(),
                &owner,
                &resolver,
                &rest,
            )
            .await
            .is_none(),
            "{} admitted an unlisted author",
            listener.name()
        );
        nip11_server.abort();
        metadata_server.abort();
    }
}

#[tokio::test]
async fn signed_dm_policy_matrix_and_revocation() {
    let (metadata_rest, metadata_server) = nip11_server(serde_json::json!([])).await;
    for listener in [ListenerBoundary::Normal, ListenerBoundary::Setup] {
        let sender = nostr::Keys::generate();
        let author = sender.public_key().to_hex();
        let agent = nostr::Keys::generate().public_key().to_hex();
        let relay = nostr::Keys::generate().public_key().to_hex();
        let (mut gate, rest, server) = connected_gate(&relay, &agent).await;
        for channel_type in ["dm", "unknown", "unexpected", "stream", "forum"] {
            let channel_id = Uuid::new_v4();
            let channel_info = pool::ChannelInfoResolver::new(
                HashMap::from([(
                    channel_id,
                    relay::ChannelInfo {
                        name: "policy test".into(),
                        channel_type: channel_type.into(),
                        description: None,
                    },
                )]),
                metadata_rest.clone(),
            );
            for mode in [
                RespondTo::Allowlist,
                RespondTo::OwnerOnly,
                RespondTo::Anyone,
                RespondTo::Nobody,
            ] {
                for principal in ["external", "owner", "sibling"] {
                    let cache = OwnerCache::new(Some(if principal == "owner" {
                        author.clone()
                    } else {
                        nostr::Keys::generate().public_key().to_hex()
                    }));
                    cache.cache_sibling(author.clone(), principal == "sibling");
                    // The same gate sees admission, removal, and re-addition.
                    for listed in [true, false, true] {
                        let allowlist = if listed {
                            HashSet::from([author.clone()])
                        } else {
                            HashSet::new()
                        };
                        let event = nostr::EventBuilder::new(
                            nostr::Kind::Custom(KIND_STREAM_MESSAGE as u16),
                            "Please reply in this conversation",
                        )
                        .tags([
                            nostr::Tag::parse(["h", &channel_id.to_string()]).unwrap(),
                            nostr::Tag::parse(["p", &agent]).unwrap(),
                        ])
                        .sign_with_keys(&sender)
                        .unwrap();
                        event.verify().unwrap();
                        let event = relay::BuzzEvent {
                            connection_generation: 0,
                            channel_id,
                            event,
                        };
                        let authorized = match listener {
                            ListenerBoundary::Normal => {
                                authorize_normal_listener_event(
                                    &mut gate,
                                    event,
                                    &mode,
                                    &allowlist,
                                    &cache,
                                    &channel_info,
                                    &rest,
                                )
                                .await
                            }
                            ListenerBoundary::Setup => {
                                setup_mode::authorize_setup_listener_event(
                                    &mut gate,
                                    event,
                                    &mode,
                                    &allowlist,
                                    &cache,
                                    &channel_info,
                                    &rest,
                                )
                                .await
                            }
                        };
                        let known = matches!(channel_type, "dm" | "stream" | "forum");
                        let expected = match mode {
                            RespondTo::Nobody => false,
                            _ if principal != "external" => true,
                            RespondTo::Allowlist => known && listed,
                            RespondTo::Anyone => matches!(channel_type, "stream" | "forum"),
                            RespondTo::OwnerOnly => false,
                        };
                        assert_eq!(
                            authorized.is_some(),
                            expected,
                            "{} {channel_type} {mode} {principal} listed={listed}",
                            listener.name()
                        );
                        if let Some(authorized) = authorized {
                            assert_eq!(authorized.into_parts().1, author);
                        }
                    }
                }
            }
        }
        server.abort();
    }
    metadata_server.abort();
}

#[tokio::test]
async fn strict_allowlist_excludes_implicit_owner_and_siblings_on_both_listeners() {
    let (metadata_rest, metadata_server) = nip11_server(serde_json::json!([])).await;
    for listener in [ListenerBoundary::Normal, ListenerBoundary::Setup] {
        let sender = nostr::Keys::generate();
        let author = sender.public_key().to_hex();
        let agent = nostr::Keys::generate().public_key().to_hex();
        let relay = nostr::Keys::generate().public_key().to_hex();
        let (rest, server) = nip11_server(serde_json::json!({ "self": relay })).await;
        let mut gate = InboundAuthorGate::connect(&rest, &agent, "strict policy test", true).await;
        for channel_type in ["dm", "stream", "unknown"] {
            let channel_id = Uuid::new_v4();
            let channel_info = pool::ChannelInfoResolver::new(
                HashMap::from([(
                    channel_id,
                    relay::ChannelInfo {
                        name: "strict policy test".into(),
                        channel_type: channel_type.into(),
                        description: None,
                    },
                )]),
                metadata_rest.clone(),
            );
            for principal in ["external", "owner", "sibling"] {
                let cache = OwnerCache::new(Some(if principal == "owner" {
                    author.clone()
                } else {
                    nostr::Keys::generate().public_key().to_hex()
                }));
                cache.cache_sibling(author.clone(), principal == "sibling");
                for listed in [true, false, true] {
                    let allowlist = if listed {
                        HashSet::from([author.clone()])
                    } else {
                        HashSet::new()
                    };
                    let event = nostr::EventBuilder::new(
                        nostr::Kind::Custom(KIND_STREAM_MESSAGE as u16),
                        "Please reply in this conversation",
                    )
                    .tags([
                        nostr::Tag::parse(["h", &channel_id.to_string()]).unwrap(),
                        nostr::Tag::parse(["p", &agent]).unwrap(),
                    ])
                    .sign_with_keys(&sender)
                    .unwrap();
                    event.verify().unwrap();
                    let event = relay::BuzzEvent {
                        connection_generation: 0,
                        channel_id,
                        event,
                    };
                    let authorized = match listener {
                        ListenerBoundary::Normal => {
                            authorize_normal_listener_event(
                                &mut gate,
                                event,
                                &RespondTo::Allowlist,
                                &allowlist,
                                &cache,
                                &channel_info,
                                &rest,
                            )
                            .await
                        }
                        ListenerBoundary::Setup => {
                            setup_mode::authorize_setup_listener_event(
                                &mut gate,
                                event,
                                &RespondTo::Allowlist,
                                &allowlist,
                                &cache,
                                &channel_info,
                                &rest,
                            )
                            .await
                        }
                    };
                    assert_eq!(
                        authorized.is_some(),
                        listed && channel_type != "unknown",
                        "{} {channel_type} {principal} listed={listed}",
                        listener.name()
                    );
                }
            }
        }
        server.abort();
    }
    metadata_server.abort();
}
