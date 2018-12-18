// Copyright (c) 2017-2018, Substratum LLC (https://substratum.net) and/or its affiliates. All rights reserved.
use sub_lib::cryptde::Key;
use neighborhood_database::NodeRecord;
use neighborhood_database::NodeRecordInner;
use neighborhood_database::NodeSignatures;
use std::collections::HashSet;

#[derive (Clone, PartialEq, Hash, Eq, Debug, Serialize, Deserialize)]
pub struct GossipNodeRecord {
    pub inner: NodeRecordInner,
    pub signatures: NodeSignatures,
}

impl GossipNodeRecord {
    pub fn from (node_record_ref: &NodeRecord, reveal_node_addr: bool) -> GossipNodeRecord {
        GossipNodeRecord {
            inner: NodeRecordInner {
                public_key: node_record_ref.public_key().clone(),
                node_addr_opt: if reveal_node_addr {
                    match node_record_ref.node_addr_opt () {
                        Some (ref node_addr) => Some ((*node_addr).clone ()),
                        None => None
                    }
                } else {
                    None
                },
                is_bootstrap_node: node_record_ref.is_bootstrap_node(),
                neighbors: node_record_ref.neighbors ().clone (),
            },
            // crashpoint
            signatures: node_record_ref.signatures().expect("Attempted to create Gossip about an unsigned NodeRecord"),
        }
    }

    pub fn to_node_record(&self) -> NodeRecord {
        let mut node_record = NodeRecord::new (
            &self.inner.public_key,
            self.inner.node_addr_opt.as_ref(),
            self.inner.is_bootstrap_node,
            Some(self.signatures.clone())
        );
        node_record.neighbors_mut ().extend (self.inner.neighbors.clone ());
        node_record
    }
}

#[derive (Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Gossip {
    pub node_records: Vec<GossipNodeRecord>,
}

pub struct GossipBuilder {
    gossip: Gossip,
    keys_so_far: HashSet<Key>,
}

impl GossipBuilder {
    pub fn new () -> GossipBuilder {
        GossipBuilder {
            gossip: Gossip {
                node_records: vec!(),
            },
            keys_so_far: HashSet::new()
        }
    }

    pub fn node (mut self, node_record_ref: &NodeRecord, reveal_node_addr: bool) -> GossipBuilder {
        if self.keys_so_far.contains (node_record_ref.public_key ()) {
            // crashpoint
            panic! ("GossipBuilder cannot add a node more than once")
        }
        if node_record_ref.signatures().is_some() {
            self.gossip.node_records.push (GossipNodeRecord::from (node_record_ref, reveal_node_addr));
            self.keys_so_far.insert (node_record_ref.public_key ().clone ());
        }
        self
    }

    pub fn build (self) -> Gossip {
        self.gossip
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use neighborhood_test_utils::make_node_record;
    use std::net::IpAddr;
    use std::str::FromStr;
    use sub_lib::node_addr::NodeAddr;

    #[test]
    #[should_panic (expected = "GossipBuilder cannot add a node more than once")]
    fn adding_node_twice_to_gossip_builder_causes_panic () {
        let node = make_node_record (1234, true, true);
        let builder = GossipBuilder::new ().node (&node, true);

        builder.node (&node, true);
    }

    #[test]
    fn adding_node_with_addr_and_reveal_results_in_node_with_addr () {
        let node = make_node_record (1234, true, false);
        let builder = GossipBuilder::new ();

        let builder = builder.node (&node, true);

        let mut gossip = builder.build ();
        assert_eq! (gossip.node_records.remove (0).inner.node_addr_opt.unwrap (), node.node_addr_opt ().unwrap ())
    }

    #[test]
    fn adding_node_with_addr_and_no_reveal_results_in_node_with_no_addr () {
        let node = make_node_record (1234, true, false);
        let builder = GossipBuilder::new ();

        let builder = builder.node (&node, false);

        let mut gossip = builder.build ();
        assert_eq! (gossip.node_records.remove (0).inner.node_addr_opt, None)
    }

    #[test]
    fn adding_node_with_no_addr_and_reveal_results_in_node_with_no_addr () {
        let node = make_node_record (1234, false, false);
        let builder = GossipBuilder::new ();

        let builder = builder.node (&node, true);

        let mut gossip = builder.build ();
        assert_eq! (gossip.node_records.remove (0).inner.node_addr_opt, None)
    }

    #[test]
    fn adding_node_with_no_addr_and_no_reveal_results_in_node_with_no_addr () {
        let node = make_node_record (1234, false, false);
        let builder = GossipBuilder::new ();

        let builder = builder.node (&node, false);

        let mut gossip = builder.build ();
        assert_eq! (gossip.node_records.remove (0).inner.node_addr_opt, None)
    }

    #[test]
    fn adding_node_with_missing_signatures_results_in_no_added_node() {
        let builder = GossipBuilder::new();

        let node = NodeRecord::new(&Key::new(&[5, 4, 3, 2]), Some(&NodeAddr::new(&IpAddr::from_str("1.2.3.4").unwrap(), &vec!(1234))), false, None);
        let builder = builder.node(&node, true);

        let gossip = builder.build();
        assert_eq!(0, gossip.node_records.len());
    }

    #[test]
    #[should_panic (expected = "Attempted to create Gossip about an unsigned NodeRecord")]
    fn gossip_node_record_cannot_be_created_from_node_with_missing_signatures() {
        let node = NodeRecord::new(&Key::new(&[5, 4, 3, 2]), Some(&NodeAddr::new(&IpAddr::from_str("1.2.3.4").unwrap(), &vec!(1234))), false, None);

        let _gossip = GossipNodeRecord::from(&node, true);
    }
}
