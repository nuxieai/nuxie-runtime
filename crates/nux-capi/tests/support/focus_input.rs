// Hand-authored focus and blur events with an optional FocusData child.
fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("focus-input fixture type");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("focus-input fixture ancestor")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .expect("focus-input fixture property")
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, body: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .expect("focus-input fixture type")
                .type_key
                .int,
        ),
    );
    body(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, property_name)));
    push_var_uint(bytes, value);
}

fn push_f32(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: f32) {
    push_var_uint(bytes, u64::from(property_key(type_name, property_name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_string(bytes: &mut Vec<u8>, kind: &str, property: &str, value: &str) {
    push_var_uint(bytes, u64::from(property_key(kind, property)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

pub fn focus_events(include_focus_data: bool) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 9658, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    push_object(&mut bytes, "Node", |bytes| {
        push_uint(bytes, "Component", "parentId", 0);
    });
    // Keep event IDs identical when removing FocusData from the file.
    for name in ["focus_arrived", "focus_left"] {
        push_object(&mut bytes, "Event", |bytes| {
            push_string(bytes, "Component", "name", name);
        });
    }
    if include_focus_data {
        push_object(&mut bytes, "FocusData", |bytes| {
            push_uint(bytes, "Component", "parentId", 1);
            push_uint(bytes, "FocusData", "focusFlags", 7);
            push_uint(bytes, "FocusData", "edgeBehaviorValue", 0);
        });
    }
    push_object(&mut bytes, "StateMachine", |_| {});
    for (kind, event) in [(13, 2), (14, 3)] {
        push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
            push_uint(bytes, "StateMachineListener", "targetId", 1);
            push_uint(
                bytes,
                "StateMachineListenerSingle",
                "listenerTypeValue",
                kind,
            );
        });
        push_object(&mut bytes, "ListenerFireEvent", |bytes| {
            push_uint(bytes, "ListenerFireEvent", "eventId", event);
        });
    }
    bytes
}
