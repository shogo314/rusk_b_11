mod support;
use support::Case;

fn source(collection: &str, callback: &str, action: &str) -> String {
    let (ty, constructor, prepare) = if collection == "Dict" {
        (
            "Dict<Key,Int>",
            "Dict::<Key,Int>()",
            "table.set(Key{value:1},2);",
        )
    } else {
        ("Set<Key>", "Set::<Key>()", "table.insert(Key{value:1});")
    };
    let (hash, equal) = if callback == "hash" {
        (
            format!("if armed{{{action}}}return 0;"),
            "return equal_int(a.value,b.value);".into(),
        )
    } else {
        (
            "return 0;".into(),
            format!("if armed{{{action}}}return equal_int(a.value,b.value);"),
        )
    };
    format!(
        r#"extern symbol state {{code: "std::function<void()> hash_change;std::function<std::int64_t()> hash_size;",}}
extern fn change() {{cpp_name:"hash_test_change",code: "void hash_test_change(Loc){{hash_change();}}",}}
extern fn size()->Int {{cpp_name:"hash_test_size",code: "Obj<std::int64_t> hash_test_size(Loc){{return make_obj<std::int64_t>(hash_size());}}",}}
extern fn release() {{cpp_name:"hash_test_release",code: "void hash_test_release(Loc){{hash_change=nullptr;hash_size=nullptr;}}",}}
extern fn watch<T>(table:T) {{code: "void @{{fn}}(Obj<@{{T}}> table,Loc loc){{hash_change=[table,loc](){{table->entries.clear();++table->version;}};hash_size=[table](){{return table->entries.size();}};}}",}}
fn equal_int(a:Int,b:Int)->Bool{{return a==b;}}
{{
struct Key{{value:Int;}}
let armed:=false;let seen:=0;
fn __hash__(key:Key)->Int{{{hash}}}
fn __eq__(a:Key,b:Key)->Bool{{{equal}}}
let table:{ty}:={constructor};watch(table);{prepare}
armed=true;wt "before";wt table.contains(Key{{value:2}});release();
}}"#
    )
}

#[test]
fn hash_and_equality_reentry_is_rejected_before_native_storage_mutation() {
    for collection in ["Dict", "Set"] {
        for callback in ["hash", "equal"] {
            let code = source(collection, callback, "change();");
            for options in [vec![], vec!["--no-opt"]] {
                let out = Case::new(&code).run_with_options("", &options);
                assert_eq!(
                    out.status.code(),
                    Some(1),
                    "{}",
                    String::from_utf8_lossy(&out.stderr)
                );
                assert_eq!(out.stdout, b"before\n");
                assert!(
                    String::from_utf8_lossy(&out.stderr)
                        .contains("container reentered during hash operation"),
                    "{}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
    }
}

#[test]
fn callbacks_can_read_size() {
    let code = source("Dict", "hash", "seen=size();").replace("release();", "wt seen;release();");
    let out = Case::new(&code).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"before\nNo\n1\n");
}

#[test]
fn callbacks_can_mutate_a_different_container() {
    for collection in ["Dict", "Set"] {
        let constructor = if collection == "Dict" {
            "Dict::<Key,Int>()"
        } else {
            "Set::<Key>()"
        };
        let code = source(collection, "hash", "change();").replace(
            "watch(table);",
            &format!("let other:={constructor};watch(other);"),
        );
        let out = Case::new(&code).run("");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"before\nNo\n");
    }
}

#[test]
fn key_exports_and_iterators_check_copy_callbacks_before_reusing_positions() {
    for collection in ["Dict", "Set"] {
        for (operation, message) in [
            ("table.keys();", "container changed during key export"),
            (
                "let it:=__iter__(table);__iter_next__(it);",
                "container changed during iteration",
            ),
        ] {
            let code=source(collection,"hash","")
                .replace("fn __hash__(key:Key)","fn __copy__(key:Key)->Key{if armed{change();}return Key{value:key.value};}fn __hash__(key:Key)")
                .replace("wt table.contains(Key{value:2});",operation);
            for options in [vec![], vec!["--no-opt"]] {
                let out = Case::new(&code).run_with_options("", &options);
                assert_eq!(
                    out.status.code(),
                    Some(1),
                    "{}",
                    String::from_utf8_lossy(&out.stderr)
                );
                assert_eq!(out.stdout, b"before\n");
                assert!(
                    String::from_utf8_lossy(&out.stderr).contains(message),
                    "{}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
    }
}

#[test]
fn copying_checks_source_mutation_from_result_hash_callbacks() {
    for collection in ["Dict", "Set"] {
        let code = source(collection, "hash", "change();")
            .replace("wt table.contains(Key{value:2});", "copy(table);");
        for options in [vec![], vec!["--no-opt"]] {
            let out = Case::new(&code).run_with_options("", &options);
            assert_eq!(
                out.status.code(),
                Some(1),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert_eq!(out.stdout, b"before\n");
            assert!(
                String::from_utf8_lossy(&out.stderr).contains("container changed during copy"),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}
