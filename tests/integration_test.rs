use std::fs::read;

use test_case::test_case;

use btf_rs::*;

fn bytes() -> Btf {
    Btf::from_bytes(&read("tests/data/vmlinux").unwrap()).unwrap()
}

fn file() -> Btf {
    Btf::from_file("tests/data/vmlinux").unwrap()
}

#[test_case(bytes())]
#[test_case(file())]
fn resolve_id_by_name(btf: Btf) {
    // Resolve primitive type.
    assert_eq!(btf.resolve_id_by_name("int").unwrap(), 9);
    // Resolve typedef.
    assert_eq!(btf.resolve_id_by_name("u64").unwrap(), 14);
    // Resolve struct.
    assert_eq!(btf.resolve_id_by_name("sk_buff").unwrap(), 1987);
    // Resolve function.
    assert_eq!(btf.resolve_id_by_name("consume_skb").unwrap(), 126822);
}

#[test_case(bytes())]
#[test_case(file())]
fn resolve_type_by_name(btf: Btf) {
    assert!(btf.resolve_type_by_name("consume_skb").is_ok());
}

#[test_case(bytes())]
#[test_case(file())]
fn resolve_type_by_name_unknown(btf: Btf) {
    assert!(btf.resolve_type_by_name("not_a_known_function").is_err());
}

#[test_case(bytes())]
#[test_case(file())]
fn check_resolved_type(btf: Btf) {
    let r#type = btf.resolve_type_by_name("sk_buff").unwrap();

    match r#type {
        Type::Struct(_) => (),
        _ => panic!("Resolved type is not a struct"),
    }
}

#[test_case(bytes())]
#[test_case(file())]
fn bijection(btf: Btf) {
    let func = match btf.resolve_type_by_name("kzalloc").unwrap() {
        Type::Func(func) => func,
        _ => panic!("Resolved type is not a function"),
    };

    assert_eq!(btf.resolve_name(&func).unwrap(), "kzalloc");

    let func_id = btf.resolve_id_by_name("kzalloc").unwrap();
    let func = match btf.resolve_type_by_id(func_id).unwrap() {
        Type::Func(func) => func,
        _ => panic!("Resolved type is not a function"),
    };

    assert_eq!(btf.resolve_name(&func).unwrap(), "kzalloc");
}

#[test_case(bytes())]
#[test_case(file())]
fn resolve_function(btf: Btf) {
    let func = match btf.resolve_type_by_name("kfree_skb_reason").unwrap() {
        Type::Func(func) => func,
        _ => panic!("Resolved type is not a function"),
    };

    assert_eq!(func.is_static(), true);
    assert_eq!(func.is_global(), false);
    assert_eq!(func.is_extern(), false);

    let proto = match btf.resolve_chained_type(&func).unwrap() {
        Type::FuncProto(proto) => proto,
        _ => panic!("Resolved type is not a function proto"),
    };

    assert_eq!(proto.parameters.len(), 2);
    assert_eq!(btf.resolve_name(&proto.parameters[0]).unwrap(), "skb");
    assert_eq!(proto.parameters[0].is_variadic(), false);
    assert_eq!(btf.resolve_name(&proto.parameters[1]).unwrap(), "reason");
    assert_eq!(proto.parameters[1].is_variadic(), false);

    match btf.resolve_type_by_id(proto.return_type_id()).unwrap() {
        Type::Void => (),
        _ => panic!("Resolved type is not void"),
    }

    let ptr = match btf.resolve_chained_type(&proto.parameters[0]).unwrap() {
        Type::Ptr(ptr) => ptr,
        _ => panic!("Resolved type is not a pointer"),
    };

    match btf.resolve_chained_type(&proto.parameters[1]).unwrap() {
        Type::Enum(_) => (),
        _ => panic!("Resolved type is not an enum"),
    }

    let r#struct = match btf.resolve_chained_type(&ptr).unwrap() {
        Type::Struct(r#struct) => r#struct,
        _ => panic!("Resolved type is not a struct"),
    };

    assert_eq!(btf.resolve_name(&r#struct).unwrap(), "sk_buff");
    assert_eq!(r#struct.size(), 232);
    assert_eq!(r#struct.members.len(), 28);

    assert_eq!(btf.resolve_name(&r#struct.members[25]).unwrap(), "truesize");

    let arg = match btf.resolve_chained_type(&r#struct.members[25]).unwrap() {
        Type::Int(int) => int,
        _ => panic!("Resolved type is not an integer"),
    };

    assert_eq!(btf.resolve_name(&arg).unwrap(), "unsigned int");
    assert_eq!(arg.is_signed(), false);
    assert_eq!(arg.is_char(), false);
    assert_eq!(arg.is_bool(), false);
}
