mod support;
use support::Case;

#[test]
fn values_distinguish_nominal_types_infer_and_forward_to_specializations() {
    let source = r#"
struct Residue<M:Int>{value:Int;}
fn residue<M:Int>(value:Int)->Residue<M>{return Residue::<M>{value:value%M};}
fn modulus<M:Int>(value:Residue<M>)->Int{return M;}
fn forward<M:Int>(value:Int)->Residue<M>{return residue::<M>(value);}
enum Flag<B:Bool>{Value(Int),Empty}
fn enabled<B:Bool>(flag:Flag<B>)->Bool{return B;}
fn repeated<N:Int>(value:Int)->Vec<Int>{return [value;N];}
let a:=forward::<7>(10);
let b:Residue<07>:=residue::<7>(5);
wt a.value,b.value,modulus(a),modulus(b),repeated::<3>(4);
let yes:=Flag::<true>::Value(1);
let no:=Flag::<false>::Empty;
wt enabled(yes),enabled(no),yes==copy(yes);
"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"3 5 7 7 4 4 4\nYes No Yes\n");
    }
}

#[test]
fn value_arguments_work_in_native_templates_local_types_and_function_values() {
    let source = r#"
extern fn constant<N:Int>()->Int{code:"Obj<std::int64_t> @{fn}(Loc){return make_obj<std::int64_t>(@{N});}",}
extern type<N:Int> Native{cpp_name:"std::array<std::int64_t,@{N}>",}
fn offset<N:Int>(x:Int)->Int{return x+N;}
fn outer<N:Int>()->Int{
    struct Local<K:Int>{value:Int;}
    fn inner<K:Int>(x:Local<K>)->Int{return K+x.value;}
    let x:=Local::<N>{value:2};
    return inner(x);
}
let unused:Native<3>;
let f:=offset::<3>;
wt constant::<-7>(),constant::<-9223372036854775808>(),f(4),outer::<5>();
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"-7 -9223372036854775808 7 7\n");
}

#[test]
fn invalid_value_arguments_and_writes_fail_before_cpp() {
    for source in [
        "struct S<N:Int>{}let x:=S::<true>();",
        "struct S<N:Bool>{}let x:=S::<3>();",
        "struct S<T>{}let x:=S::<3>();",
        "struct S<N:Int>{}let x:S<9223372036854775808>;",
        "fn f<N:Str>(){}",
        "fn f<N:Int>(N:Int){}",
        "fn f<N:Int>(){N+=1;}f::<3>();",
        "fn f<N:Int>(){N:=1;}f::<3>();",
        "struct S<N:Int>{}let a:=S::<3>();let b:S<4>:=a;",
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
fn formatting_and_api_signatures_keep_value_kinds_and_negative_arguments() {
    use std::{fs, process::Command};
    let case = Case::new(
        "/// Fixed capacity.\nstruct Slots<T,N:Int,B:Bool>{value:T;}fn offset<N:Int>(x:Int)->Int{return x+N;}wt offset::<-7>(10);",
    );
    let formatted = Command::new(env!("CARGO_BIN_EXE_rusk"))
        .current_dir(&case.0)
        .args(["fmt", "main.rusk"])
        .output()
        .unwrap();
    assert!(
        formatted.status.success(),
        "{}",
        String::from_utf8_lossy(&formatted.stderr)
    );
    let text = String::from_utf8(formatted.stdout).unwrap();
    assert!(text.contains("Slots<T, N: Int, B: Bool>"), "{text}");
    assert!(text.contains("offset::<-7>"), "{text}");
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
    let slots = data["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "Slots")
        .unwrap();
    assert_eq!(
        slots["type_parameters"],
        serde_json::json!(["T", "N: Int", "B: Bool"])
    );
    assert_eq!(case.run("").stdout, b"3\n");
}

#[test]
fn modular_operations_and_empty_value_tags_compose_with_automatic_operations() {
    let source = include_str!("../examples/beta11/value_arguments.rusk");
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"3 5 1 7 Yes\n");
    }
}
