//! Group from Layers (`layer.new.groupFromLayers`) refuses the Background (#14709).

use super::*;

/// A 24×16 document: the Background and a layer "Red" above it, which is active.
fn new_session() -> (Session, LayerId, LayerId) {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 24, "height": 16})).unwrap();
    let background = s.active().unwrap().doc.background().expect("a new document has a Background").id;
    let red = LayerId(s.execute("layer.new.layer", json!({"name": "Red"})).unwrap()["layer"].as_u64().unwrap());
    (s, background, red)
}

/// The layer tree, the selection, the active layer and the number of history states.
fn snapshot(s: &Session) -> (Document, Vec<LayerId>, Option<LayerId>, usize) {
    let st = s.active().unwrap();
    ((*st.doc).clone(), st.selected_layers(), st.active_layer, st.history.entries().len())
}

/// The command fails with `params` and leaves the document, selection and history as they were.
fn refused(s: &mut Session, cmd: &str, params: Value) {
    let before = snapshot(s);
    assert!(s.execute(cmd, params.clone()).is_err(), "{cmd} {params} must be refused");
    assert!(before == snapshot(s), "{cmd} {params} must change nothing");
}

#[test]
fn background_alone_is_refused() {
    let (mut s, background, _) = new_session();
    s.execute("layer.select", json!({"layer": background.0})).unwrap();
    assert!(!s.is_enabled("layer.new.groupFromLayers"));
    refused(&mut s, "layer.new.groupFromLayers", json!({}));
    assert!(s.active().unwrap().doc.is_background(background));
}

#[test]
fn background_with_another_layer_is_refused() {
    let (mut s, background, red) = new_session();
    s.execute("layer.select", json!({"layer": background.0})).unwrap();
    s.execute("layer.select", json!({"layer": red.0, "mode": "add"})).unwrap();
    assert_eq!(s.active().unwrap().selected_layers().len(), 2);
    assert!(!s.is_enabled("layer.new.groupFromLayers"));
    refused(&mut s, "layer.new.groupFromLayers", json!({"name": "Pair"}));
    assert!(s.active().unwrap().doc.is_background(background));
}

#[test]
fn explicit_background_is_refused_when_another_layer_is_active() {
    let (mut s, background, red) = new_session();
    assert_eq!(s.active().unwrap().active_layer, Some(red));
    refused(&mut s, "layer.new.groupFromLayers", json!({"layer": background.0}));
    assert!(s.active().unwrap().doc.is_background(background));
}

/// A document whose only layer is the Background, as an Indexed Color document always is.
#[test]
fn background_only_document_is_refused() {
    let mut s = Session::new();
    s.execute("file.new", json!({"width": 24, "height": 16})).unwrap();
    assert_eq!(s.active().unwrap().doc.layer_count(), 1);
    assert!(s.active().unwrap().doc.background().is_some());
    assert!(!s.is_enabled("layer.new.groupFromLayers"));
    refused(&mut s, "layer.new.groupFromLayers", json!({}));
}

#[test]
fn layers_other_than_the_background_still_group() {
    let (mut s, background, red) = new_session();
    assert!(s.is_enabled("layer.new.groupFromLayers"));
    let g = LayerId(s.execute("layer.new.groupFromLayers", json!({})).unwrap()["layer"].as_u64().unwrap());
    let doc = &s.active().unwrap().doc;
    assert_eq!(doc.layer(g).unwrap().children().unwrap().iter().map(|l| l.id).collect::<Vec<_>>(), vec![red]);
    assert!(doc.is_background(background));
}

/// Group Layers with the Background was not measured, so it keeps grouping it.
#[test]
fn group_layers_keeps_grouping_the_background() {
    let (mut s, background, _) = new_session();
    s.execute("layer.select", json!({"layer": background.0})).unwrap();
    let g = LayerId(s.execute("layer.groupLayers", json!({})).unwrap()["layer"].as_u64().unwrap());
    let doc = &s.active().unwrap().doc;
    assert_eq!(doc.layer(g).unwrap().children().unwrap().iter().map(|l| l.id).collect::<Vec<_>>(), vec![background]);
}
