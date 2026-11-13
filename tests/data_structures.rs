mod support;
use support::Case;

#[test]
fn generic_data_structures_work_with_int_and_user_types() {
    let source = String::from(
        r#"
struct Weight { value:Int; }
fn __add__(a:Weight,b:Weight)->Weight { return Weight { value:a.value+b.value }; }
fn __sub__(a:Weight,b:Weight)->Weight { return Weight { value:a.value-b.value }; }
let values:=[1,2,3];
let fw:=FenwickTree(values,0); let seg:=SumSegTree(values,0);
fw.apply(1,5); seg.set(1,7); wt fw.prod(0,3),seg.prod(0,3),values;
let zero:=Weight { value:0 };
let weights:=[Weight { value:2 },Weight { value:3 }];
let generic_fw:=FenwickTree(weights,zero); let generic_seg:=SumSegTree(weights,zero);
generic_fw.apply(0,Weight { value:4 }); generic_seg.set(0,Weight { value:6 });
wt generic_fw.prod(0,2).value,generic_seg.prod(0,2).value,zero.value,weights[0].value;
let detached = generic_fw; detached.apply(0,Weight { value:1 });
wt generic_fw.prefix_prod(1).value,detached.prefix_prod(1).value;
let strings:=SumSegTree(["a","b","c","d","e"],"");
wt strings.prod(1,5); strings.set(2,"X"); wt strings.prod(0,5);
let empty:=FenwickTree::<Int>(0,0); wt empty.prod(0,0);
let empty_seg:=SumSegTree::<Int>([],0); wt empty_seg.prod(0,0);
"#,
    );
    let out = Case::new(&source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"11 11 1 2 3\n9 9 0 2\n6 7\nbcde\nabXde\n0\n0\n"
    );
}

#[test]
fn data_structure_queries_match_a_naive_model() {
    let source = String::from(
        r#"
in n,q:Int;
in initial(n):Vec<Int>;
let fw:=FenwickTree(initial,0); let seg:=SumSegTree(initial,0);
for query in 0..q {
    in kind,left,right:Int;
    if kind==0 { fw.apply(left,right); seg.set(left,seg.prod(left,left+1)+right); }
    else { wt fw.prod(left,right),seg.prod(left,right); }
}
"#,
    );
    let mut values = [0i64; 17];
    let mut input = String::from("17 200\n0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n");
    let mut expected = String::new();
    let mut seed = 314u64;
    for query in 0..200 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let a = ((seed >> 32) % 17) as usize;
        if query % 3 == 0 {
            let delta = (seed % 21) as i64 - 10;
            values[a] += delta;
            input.push_str(&format!("0 {a} {delta}\n"));
        } else {
            let b = (seed % 18) as usize;
            let left = a.min(b);
            let right = a.max(b);
            let sum: i64 = values[left..right].iter().sum();
            input.push_str(&format!("1 {left} {right}\n"));
            expected.push_str(&format!("{sum} {sum}\n"));
        }
    }
    let out = Case::new(&source).run(&input);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, expected.as_bytes());
}

#[test]
fn invalid_data_structure_indices_fail_instead_of_looping() {
    for statement in [
        "let fw:=FenwickTree(2,0); fw.apply(-1,1);",
        "let fw:=FenwickTree(2,0); fw.apply(2,1);",
        "let fw:=FenwickTree(2,0); fw.prefix_prod(3);",
        "let fw:=FenwickTree(2,0); fw.prod(2,1);",
        "let seg:=SumSegTree([1,2],0); seg.set(2,0);",
        "let seg:=SumSegTree([1,2],0); seg.prod(0,3);",
    ] {
        let out = Case::new(&String::from(statement)).run("");
        assert_eq!(out.status.code(), Some(1), "{statement}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("assertion failed"));
    }
}

#[test]
fn bit_operations_keep_integer_values_independent() {
    let out = Case::new("let a:=6; let b:=3; wt a&b,a|b,a^b,(-1)&8,1|2^3&6,a,b; assert((a&b)==2);")
        .run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"2 7 5 8 1 6 3\n");
}

#[test]
fn migrated_algorithms_work_without_includes_and_preserve_inputs() {
    let code = r##"
let text:="aaaaa";wt z_algorithm(text),text;
wt z_algorithm("").len(),z_algorithm("abacaba");
wt longest_common_subsequence("abcbdab","bdcaba"),longest_common_subsequence("","a");
wt pow_mod(-2,3,5),pow_mod(0,0,1),pow_mod(9223372036854775806,2,9223372036854775807);
let grid:=["...",".#.","..."];wt grid_shortest_path(grid,3,3),grid[1];
wt grid_shortest_path([".#","#."],2,2);
wt is_prime(-1),is_prime(0),is_prime(1),is_prime(2),is_prime(3),is_prime(9),is_prime(2147483647),is_prime(9223372036854775807);
"##;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(code).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.stdout,
            b"5 4 3 2 1 aaaaa\n0 7 0 1 0 3 0 1\n4 0\n2 0 1\n4 .#.\n-1\nNo No No Yes Yes No Yes No\n"
        );
    }
}

#[test]
fn migrated_algorithm_preconditions_fail_with_source_locations() {
    for code in [
        "pow_mod(2,-1,5);",
        "pow_mod(2,3,0);",
        "grid_shortest_path([],0,0);",
        "grid_shortest_path([\"..\",\".\"],2,2);",
    ] {
        let out = Case::new(code).run("");
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("assertion failed"));
    }
}
