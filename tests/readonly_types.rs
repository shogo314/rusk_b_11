mod support;
use support::Case;

#[test]
fn readonly_return_types_share_updates_and_allow_independent_copies() {
    let source = r#"
fn view(value:Int)->readonly Int{return value;}
fn first(values:Vec<Int>)->readonly Int{return values[0];}
let original:=1;let a:=view(original);original=2;
let detached=a;detached+=3;
let values:=[3,4];let b:=first(values);values[0]=5;
let explicit:readonly Int:=7;wt a,detached,b,explicit+1;
explicit:=9;wt explicit;
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"2 5 5 8\n9\n");
}

#[test]
fn readonly_elements_leave_the_outer_container_mutable() {
    let source = r#"
let values:Vec<readonly Int>:=[1,2];
values.push_back(3);values[0]:=4;
wt values.len(),values[0],values.pop_back();
values.clear();wt values.len();
struct Row{left:readonly Int;right:Int;}
let row:=Row{left:5,right:6};row.left:=7;row.right=8;wt row.left,row.right;
let pair:(readonly Int,Int):=(9,10);pair[1]=11;wt pair[0],pair[1];
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"3 4 3\n0\n7 8\n9 11\n");
}

#[test]
fn readonly_types_reject_writes_aliases_and_invariant_container_conversions() {
    for source in [
        "fn unit(){}let x:readonly Unit:=unit();x=unit();",
        "fn forbid<Unit>(x:readonly Unit){x=2;}forbid::<Int>(1);",
        "fn view(x:Int)->readonly Int{return x;}view(1)=2;",
        "let x:readonly Int:=1;x+=1;",
        "let x:readonly Int; x:=1; x:=2; x=3;",
        "fn f<T>(x:T)->readonly T{return x;}let a:=f(1);let b:=a;b=2;",
        "fn f(x:readonly Int){x=1;}f(2);",
        "let a:Vec<readonly Int>:=[1];a[0]=2;",
        "struct S{x:readonly Int;}let s:=S{x:1};s.x=2;",
        "fn both<T>(a:Vec<T>,b:Vec<T>){}let a:Vec<readonly Int>:=[1];let b:Vec<Int>:=[2];both(a,b);",
        "fn both<T>(a:Vec<T>,b:Vec<T>){}let a:Vec<Int>:=[1];let b:Vec<readonly Int>:=[2];both(a,b);",
        "fn identity<T>(x:Vec<T>)->Vec<T>{return x;}let a:Vec<Int>:=[1];let b:Vec<readonly Int>:=identity(a);",
        "let a:Vec<Int>:=[1];let b:Vec<readonly Int>:=a;",
        "let a:Vec<readonly Int>:=[1];let b:Vec<Int>:=a;",
    ] {
        let out = Case::new(source).compile();
        assert!(!out.status.success(), "{source}");
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("internal compiler error"),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn mutable_containers_can_store_already_readonly_aliases_and_mutate_only_slots() {
    let source = r#"
fn view(x:Int)->readonly Int{return x;}
fn append<T>(xs:Vec<T>,x:T){xs.push_back(x);}
readonly let a:=1;
let xs:Vec<readonly Int>:=[a,view(2)];
append(xs,view(3));xs[0]:=view(4);
wt xs.len(),xs.pop_back();xs.clear();wt xs.len();
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"3 3\n0\n");
}

#[test]
fn readonly_nominal_members_and_captures_do_not_freeze_unrelated_writable_fields() {
    let source = r#"
struct Row{left:readonly Int;right:Int;}
fn bump(row:Row){row.right+=1;}
let row:=Row{left:1,right:2};bump(row);
let a:readonly Int:=3;let counter:=0;
let f:=(x:Int)=>{counter+=1;a+x};wt row.left,row.right,f(4),counter;
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"1 3 7 1\n");
}

#[test]
fn readonly_native_returns_and_members_are_checked_without_caller_modifiers() {
    for source in [
        r#"extern fn alter<T>(x:T){code:"void @{fn}(Obj<@{T}> x,Loc){*x=9;}",}alter::<readonly Int>(1);"#,
        r#"extern fn view(x:Int)->readonly Int{cpp_name:"view",code:"Obj<std::int64_t> view(Obj<std::int64_t> x,Loc){return x;}",}view(1)=2;"#,
        r#"extern fn alias(x:readonly Int)->Int{cpp_name:"alias",code:"Obj<std::int64_t> alias(Obj<std::int64_t> x,Loc){return x;}",}alias(1)=2;"#,
        r#"extern fn alter<T>(xs:Vec<T>){code:"void @{fn}(Obj<@{Vec<T>}> xs,Loc){*(*xs)[0]=9;}",}let xs:Vec<readonly Int>:=[1];alter(xs);"#,
        r#"struct S{x:readonly Int;}extern fn alter<T>(x:T){code:"void @{fn}(Obj<@{T}> x,Loc){*x->field_0=9;}",}alter(S{x:1});"#,
    ] {
        let out = Case::new(source).compile();
        assert!(!out.status.success(), "{source}");
        let error = String::from_utf8_lossy(&out.stderr);
        assert!(error.contains("readonly"), "{error}");
        assert!(!error.contains("internal compiler error"), "{error}");
    }
}

#[test]
fn readonly_members_cannot_escape_as_writable_unknown_native_aliases() {
    let source = r#"
extern fn alias<T>(readonly xs:Vec<T>)->Vec<Int>{
    code:"Obj<@{Vec<Int>}> @{fn}(Obj<@{Vec<T>}> xs,Loc){return xs;}",
}
let xs:Vec<readonly Int>:=[1];let unqualified:=alias(xs);unqualified[0]=9;
"#;
    let out = Case::new(source).compile();
    assert!(!out.status.success());
    let error = String::from_utf8_lossy(&out.stderr);
    assert!(error.contains("readonly"), "{error}");
}

#[test]
fn native_children_of_automatic_operations_cannot_mutate_readonly_fields() {
    for operation in ["let snapshot=values;", "wt values==values;"] {
        let source = format!(
            r#"
struct Cell{{value:Int;}}
fn __copy__(a:Cell)->Cell{{a.value+=1;return Cell{{value:a.value}};}}
fn __eq__(a:Cell,b:Cell)->Bool{{a.value+=1;return a.value==b.value;}}
let values:Vec<readonly Cell>:=[Cell{{value:1}}];
{operation}
"#
        );
        let out = Case::new(&source).compile();
        assert!(!out.status.success(), "{operation}");
        let error = String::from_utf8_lossy(&out.stderr);
        assert!(error.contains("readonly"), "{error}");
    }
}

#[test]
fn readonly_generic_types_are_canonical_and_unit_keeps_its_object_abi() {
    let source = r#"
struct Holder<T>{value:T;}
fn hold<T>(x:T)->Holder<readonly T>{return Holder::<readonly T>{value:x};}
let a:Holder<readonly Int>:=hold::<readonly Int>(1);
let b:readonly readonly Int:=2;wt a.value,b;
fn view<T>(x:T)->readonly T{return x;}
fn unit(){}
let units:Vec<readonly Unit>:=[unit(),view(unit())];
units.pop_back();units.clear();wt units.len();
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"1 2\n0\n");
}

#[test]
fn empty_repeated_nested_and_readonly_outer_arrays_keep_their_qualifiers() {
    let source = r#"
let empty:readonly Vec<Int>:=[];
let repeated:Vec<readonly Int>:=[2;3];repeated.push_back(4);
let nested:Vec<readonly Vec<Int>>:=[[1,2],[3]];
let saved:=nested[0];nested.push_back([4,5]);
let (a,b):(readonly Int,Int):=(5,6);b=7;
wt empty.len(),repeated,nested.len(),saved,a,b;
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"0 2 2 2 4 3 1 2 5 7\n");
}

#[test]
fn readonly_example_preserves_sharing_copies_and_contained_permissions_in_both_modes() {
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(include_str!("../examples/beta11/readonly_types.rusk"))
            .run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"9 4 9 5 3\n3 0\n9 7 10 1\n");
    }
}

#[test]
fn readonly_type_spelling_is_preserved_by_formatting_search_and_api_documents() {
    use std::{fs, process::Command};
    let case = Case::new(
        "/// A read-only getter.\nfn get<T>(x:readonly Vec<T>)->readonly T{return x[0];}let xs:Vec<readonly Int>:=[1];wt xs[0];",
    );
    let fmt = Command::new(env!("CARGO_BIN_EXE_rusk"))
        .current_dir(&case.0)
        .args(["fmt", "main.rusk"])
        .output()
        .unwrap();
    assert!(
        fmt.status.success(),
        "{}",
        String::from_utf8_lossy(&fmt.stderr)
    );
    let text = String::from_utf8(fmt.stdout).unwrap();
    assert!(text.contains("x: readonly Vec<T>"), "{text}");
    assert!(text.contains("-> readonly T"), "{text}");
    fs::write(case.0.join("main.rusk"), &text).unwrap();
    let api = Command::new(env!("CARGO_BIN_EXE_rusk"))
        .current_dir(&case.0)
        .args(["doc", "--file", "main.rusk", "--json"])
        .output()
        .unwrap();
    assert!(
        api.status.success(),
        "{}",
        String::from_utf8_lossy(&api.stderr)
    );
    let data: serde_json::Value = serde_json::from_slice(&api.stdout).unwrap();
    let getter = data["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "get")
        .unwrap();
    assert_eq!(getter["return_type"], "readonly T");
    assert_eq!(getter["parameters"][0]["type"], "readonly Vec<T>");
    assert!(
        !getter["signature"]
            .as_str()
            .unwrap()
            .contains("__RuskReadonly")
    );
    assert_eq!(Case::new(&text).run("").stdout, b"1\n");
}

#[test]
fn enum_payloads_and_record_patterns_keep_readonly_access() {
    let source = r#"
struct Cell{value:Int;}
enum Choice{Empty,Value(readonly Cell)}
let original:=Cell{value:1};let choice:=Choice::Value(original);
original.value=2;
match choice{Choice::Empty=>{wt 0;}Choice::Value(Cell{value})=>{wt value;}}
fn view<T>(x:T)->readonly T{return x;}
if let Cell{value} = view(original){wt value;}
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"2\n2\n");
    for source in [
        "enum E{Value(readonly Int)}let e:=E::Value(1);if let E::Value(x)=e{x=2;}",
        "struct C{x:Int;}enum E{Value(readonly C)}let e:=E::Value(C{x:1});if let E::Value(C{x})=e{x=2;}",
    ] {
        let out = Case::new(source).compile();
        assert!(!out.status.success(), "{source}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("readonly"),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn repeated_readonly_type_prefixes_have_a_source_limit_before_recursive_cst_cloning() {
    let source = format!("let x: {}Int := 1;", "readonly ".repeat(129));
    let out = Case::new(&source).compile();
    assert!(!out.status.success());
    let error = String::from_utf8_lossy(&out.stderr);
    assert!(error.contains("readonly type nesting limit"), "{error}");
    assert!(!error.contains("internal compiler error"));
}

#[test]
fn nested_readonly_values_keep_outer_storage_mutable() {
    let code = r#"
fn append<T>(xs:Vec<T>,value:T){xs.push_back(value);}
let xs:Vec<Option<readonly Int>>:=[];
append(xs,Some::<readonly Int>(1));
xs.push_back(None::<readonly Int>());
wt xs.len();xs.clear();wt xs.len();
struct Tree {value: Int;children:Vec<Tree>;}
fn view(tree:Tree)->readonly Tree{return tree;}
let tree:=Tree{value:1,children:[]};tree.children.push_back(Tree{value:2,children:[]});
let duplicate=copy(view(tree));duplicate.value=3;wt tree.value,duplicate.value;
"#;
    let out = Case::new(code).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"2\n0\n1 3\n");
}

#[test]
fn recursive_native_contract_proofs_do_not_hide_mutating_or_aliasing_leaves() {
    for implementation in ["x.value+=1;return Leaf{value:x.value};", "return x;"] {
        let code = format!(
            r#"
struct Leaf{{value:Int;}}
fn __copy__(x:Leaf)->Leaf{{{implementation}}}
struct Tree{{leaf:Leaf;children:Vec<Tree>;}}
fn view(x:Tree)->readonly Tree{{return x;}}
let tree:=Tree{{leaf:Leaf{{value:1}},children:[]}};
let duplicate=copy(view(tree));duplicate.leaf.value=3;
"#
        );
        let out = Case::new(&code).compile();
        assert!(!out.status.success(), "{code}");
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("internal compiler error"),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn typed_retention_does_not_seal_extra_dynamic_readonly_members() {
    for call in ["xs.push_back(row);", "append(xs,row);"] {
        let source = format!(
            r#"
fn view(x:Int)->readonly Int{{return x;}}
fn append<T>(xs:Vec<T>,x:T){{xs.push_back(x);}}
struct Row{{left:readonly Int;right:Int;}}
let row:=Row{{left:1,right:view(2)}};
let xs:Vec<Row>:=[];{call}xs[0].right=3;
"#
        );
        let out = Case::new(&source).compile();
        assert!(!out.status.success(), "{source}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("readonly"),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
