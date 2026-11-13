mod support;
use support::Case;
#[test]
fn permutation_enumeration_is_lazy_independent_and_unique() {
    let source = r#"
let input:=[2,1,1];let it:=permutations(input);let saved:Vec<Vec<Int>>:=[];
for p in it {saved.push_back(p);wt p;}
wt input,saved.len();saved[0][0]=9;wt saved[1];
let empty:Vec<Int>:=[];wt permutations(empty).to_vec().len();
let a:=[1,2,3];wt next_permutation(a),a;wt prev_permutation(a),a;wt prev_permutation(a),a;
let original:=permutations([1,2,3]);__iter_next__(original);let detached=original;
wt __iter_next__(original),__iter_next__(detached);
"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options("", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout,b"1 1 2\n1 2 1\n2 1 1\n2 1 1 3\n1 2 1\n1\nYes 1 3 2\nYes 1 2 3\nNo 3 2 1\n1 3 2 1 3 2\n");
    }
}

#[test]
fn big_integers_use_floor_arithmetic_and_independent_copies() {
    let out=Case::new(r#"let a:=BigInt("123456789012345678901234567890");let b=a;a+=BigInt(1);wt a,b;wt BigInt(-7)/BigInt(3),BigInt(-7)%BigInt(3),BigInt(7)/BigInt(-3),BigInt(7)%BigInt(-3);wt BigInt(2)**100;wt BigInt(-3)>>1,BigInt(-3)&BigInt(6),~BigInt(0);wt Int(BigInt("-9223372036854775808"));"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout,b"123456789012345678901234567891 123456789012345678901234567890\n-3 2 -3 -2\n1267650600228229401496703205376\n-2 4 -1\n-9223372036854775808\n");
}

#[test]
fn packed_arrays_and_bits_have_value_reads_and_independent_copies() {
    let source = r#"
let a:=IntArray([3,1,2]);let held:=a.get(0);a[0]:=4;a.push_back(9);let b=a;a.sort();wt a,b,held,a.sum();
let xs:=a.map((x:Int)=>x*2).to_vec();wt xs;
let bits:=BitVec("10101");bits[1]:=true;wt bits,bits.count(),bits<<2,~bits;
let fixed_bits:=Bits::<5>("00101");wt fixed_bits,fixed_bits>>1,fixed_bits.len(),fixed_bits.to_vec();
"#;
    let out = Case::new(source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"1 2 4 9 4 1 2 9 3 16\n2 4 8 18\n10111 4 11100 01000\n00101 00010 5 Yes No Yes No No\n"
    );
}

#[test]
fn deque_and_fixed_array_preserve_shared_elements_and_copy_state() {
    let out=Case::new(r#"
let q:=Deque([1,2]);let value:=3;q.push_front(value);value=4;q.push_back(5);wt q.to_vec(),q.pop_front(),q.pop_back();
let copied=q;q[0]=7;wt q.to_vec(),copied.to_vec();
let a:=Array::<Int,3>([1,2,3]);let held:=a[0];a[0]=9;wt held,a.to_vec();
let b=a;a[1]:=8;wt a.to_vec(),b.to_vec();
let filled:=Array::<Int,2>(4);filled[0]=5;wt filled.to_vec();
fn unit(){}let units:=Deque::<Unit>();units.push_back(unit());units.pop_front();wt units.len();
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"4 1 2 5 4 5\n7 2 1 2\n9 9 2 3\n9 8 3 9 2 3\n5 4\n0\n"
    );
}

#[test]
fn ordered_sets_multisets_and_maps_iterate_by_key() {
    let out=Case::new(r#"
let s:=OrderedSet::<Int>();wt s.insert(3),s.insert(1),s.insert(3);wt s.to_vec(),s.lower_bound(2)??9,s.upper_bound(3)??9;
let m:=MultiSet::<Int>();m.insert(2);m.insert(1);m.insert(2);wt m.to_vec(),m.count(2),m.erase_one(2),m.to_vec();
let d:=OrderedDict::<Int,Str>();d.set(3,"c");d.set(1,"a");let held:=d.get(1)??"?";d[1]="A";wt d.keys(),d.values(),held;
let c=d;c[1]="X";wt d.values(),c.values();
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"Yes Yes No\n1 3 3 9\n1 2 2 2 Yes 1 2\n1 3 A c A\nA c X c\n"
    );
}

#[test]
fn selection_sorted_sets_and_numeric_algorithms_match_expected_results() {
    let out=Case::new(r#"
let a:=[5,1,4,2,3];nth_element(a,2);wt a[2];partial_sort(a,3);wt a[0],a[1],a[2];
let b:=[1,1,2,2,3];wt unique(b),b;rotate(b,1);wt b;
let c:=[1,2,3,4];wt partition(c,(n:Int)=>n%2==0),c;
wt merge_sorted([1,3],[2,3]),set_union([1,1,3],[1,2]),set_intersection([1,1,3],[1,2]),set_difference([1,1,3],[1,2]);
wt partial_sum([1,2,3]),adjacent_difference([1,3,6]),inner_product([1,2],[3,4],5);
wt gcd(-9223372036854775808,2),lcm(-6,8),gcd(BigInt("12345678901234567890"),BigInt(30));
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"3\n1 2 3\n3 1 2 3\n2 3 1\n2 2 4 1 3\n1 2 3 3 1 1 2 3 1 1 3\n1 3 6 1 2 3 16\n2 24 30\n"
    );
}

#[test]
fn slices_share_updates_copy_the_owner_and_keep_iterator_positions_independent() {
    let out=Case::new(r#"
let source:=[1,2,3,4];let view:=slice(source,1,3);let readonly_view:=readonly_slice(view);
source[1]=8;wt view.to_vec(),readonly_view.copy_to_vec();view[1]:=9;wt source;
let detached=view;source[1]=7;wt detached.to_vec(),view.to_vec();
let it:=__iter__(view);__iter_next__(it);let independent=it;wt __iter_next__(it),__iter_next__(independent);
let packed:=IntArray([3,4,5]);let part:=slice(packed,1,3);part[0]:=6;wt packed,part.to_vec();
let q:=Deque([1,2,3]);let qview:=slice(q,1,3);qview[0]=7;wt q.to_vec();
let a:=Array::<Int,2>([1,2]);wt slice(a).to_vec();
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"8 3 8 3\n1 8 9 4\n8 9 7 9\n9 9\n3 6 5 6 5\n1 7 3\n1 2\n"
    );
}

#[test]
fn bigint_arithmetic_matches_python_integer_fixtures() {
    // Expected values were computed with Python integers, independently of the
    // word representation and division algorithm in the runtime.
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/bigint-arithmetic.json")).unwrap();
    let mut input = format!("{}\n", cases.len());
    let mut expected = String::new();
    for case in cases {
        input.push_str(case["input"].as_str().unwrap());
        expected.push_str(case["output"].as_str().unwrap());
    }
    let source = r#"in count:Int;for i in 0..count{in left,right:Str;in shift:Int;let a:=BigInt(left);let b:=BigInt(right);wt a+b,a-b,a*b,a/b,a%b,a&b,a|b,a^b,~a,a<<shift,a>>shift,Str(a),a==b,a<b;}"#;
    for options in [vec![], vec!["--no-opt"]] {
        let out = Case::new(source).run_with_options(&input, &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, expected.as_bytes());
    }
}

#[test]
fn scalar_conversions_hashes_and_static_lengths_are_consistent() {
    let out=Case::new(r#"
let limit:=Int128("-170141183460469231731687303715884105728");wt Int128(BigInt(limit)),gcd(limit,Int128(2));
let dict:=Dict::<BigInt,Int>();dict.set(BigInt("00012"),7);wt dict.get(BigInt(12))??0,!BigInt(0),Bool(BigInt(-1));
let a:=Bits::<0>(false);wt a.len(),a.count(),a.all(),a.to_vec().len();
let empty:Vec<Int>:=[];let fixed_array:=Array::<Int,0>(empty);wt fixed_array.len();
"#).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout,
        b"-170141183460469231731687303715884105728 2\n7 Yes Yes\n0 0 Yes 0\n0\n"
    );
}

#[test]
fn invalid_indices_lengths_and_arithmetic_fail_before_native_undefined_behavior() {
    for (source, message) in [
        ("Int(BigInt(2)**100);", "integer conversion out of range"),
        ("Int128(BigInt(2)**127);", "integer conversion out of range"),
        ("BigInt(1)/BigInt(0);", "division by zero"),
        ("BigInt(1)%BigInt(0);", "division by zero"),
        ("BigInt(\"1x\");", "invalid BigInt input"),
        ("BigInt(2)**-1;", "negative integer exponent"),
        ("let a:=IntArray([1]);a[1];", "array index out of range"),
        ("IntArray(-1,0);", "negative array length"),
        ("Bits::<3>(\"11\");", "bit length mismatch"),
        ("Bits::<-1>(false);", "negative bit length"),
        ("BitVec(\"12\");", "invalid bit string"),
        ("BitVec(\"1\")&BitVec(\"11\");", "bit length mismatch"),
        ("Array::<Int,2>([1]);", "array length mismatch"),
        ("Array::<Int,-1>(0);", "negative array length"),
        ("let q:=Deque::<Int>();q.pop_front();", "empty deque"),
        (
            "let values:=[1];let s:=slice(values);values.push_back(2);s.len();",
            "assertion failed",
        ),
        (
            "let a:=[1];nth_element(a,2);",
            "selection index out of range",
        ),
        ("lcm(9223372036854775807,2);", "integer overflow"),
        (
            "let it:=permutations([1]);__iter_next__(it);__iter_next__(it);",
            "assertion failed",
        ),
    ] {
        let out = Case::new(source).run("");
        assert_eq!(
            out.status.code(),
            Some(1),
            "{source}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(message),
            "{source}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn readonly_and_value_reads_cannot_silently_mutate_copies() {
    for source in [
        "let a:=IntArray([1]);a[0]=2;",
        "let bits:=BitVec(\"1\");bits[0]=false;",
        "readonly let a:=IntArray([1]);a.set(0,2);",
        "readonly let n:=BigInt(3);n+=BigInt(1);",
        "readonly let q:=Deque([1]);q.push_back(2);",
        "let a:=[1];let view:=readonly_slice(a);view[0]=2;",
        "let a:=[1];let view:=readonly_slice(a);view[0]:=2;",
        "let a:=Array::<Int,2>([1,2]);let b:Array<Int,3>:=a;",
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
fn mutation_during_lazy_iteration_and_callbacks_is_detected() {
    for (source, message) in [
        (
            "let a:=IntArray([1,2]);let it:=a.map((x:Int)=>{a.push_back(3);x});__iter_next__(it);",
            "container changed during iteration",
        ),
        (
            "let a:=Deque([1,2]);let it:=__iter__(a);a.push_front(3);__iter_next__(it);",
            "container changed during iteration",
        ),
        (
            "let a:=Array::<Int,2>([1,2]);let it:=__iter__(a);a=Array::<Int,2>([3,4]);__iter_next__(it);",
            "container changed during iteration",
        ),
        (
            "let a:=[1,2];next_permutation_by_comp(a,(x:Int,y:Int)=>{a.push_back(3);x<y});",
            "assertion failed",
        ),
        (
            "let a:=[1,2];partition(a,(x:Int)=>{a.push_back(3);true});",
            "container changed during callback",
        ),
    ] {
        let out = Case::new(source).run("");
        assert_eq!(
            out.status.code(),
            Some(1),
            "{source}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(message),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn standard_library_example() {
    let out = Case::new(include_str!("../examples/beta11/standard_library.rusk")).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"1 1 2\n1 2 1\n2 1 1\n2 1 1\n1234567890123456789012345678900 -3 2\n1 2 4 7\n01010 3\n1 2 3 3\n1 8 3 8 3\n1 3 3 2\n1 2 a b\n6 72 1 3 6\n");
}

#[test]
fn ordered_comparators_cannot_reenter_native_trees() {
    let source = include_str!("../examples/beta11/ordered_reentry.rusk");
    for constructor in [
        "OrderedSet::<Key>()",
        "MultiSet::<Key>()",
        "OrderedDict::<Key,Int>()",
    ] {
        let code = source.replace("OrderedSet::<Key>()", constructor);
        let code = if constructor.contains("Dict") {
            code.replace("table.insert(", "table.set(").replace(
                "value: 1\n        }\n    );",
                "value: 1\n        }, 3\n    );",
            )
        } else {
            code
        };
        let out = Case::new(&code).run("");
        assert_eq!(
            out.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, b"before\n");
        assert!(
            String::from_utf8_lossy(&out.stderr)
                .contains("container reentered during ordered operation"),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn packed_bit_operations_cross_word_boundaries_and_mask_the_tail() {
    let mut source = String::new();
    let mut expected = String::new();
    for n in [0usize, 1, 63, 64, 65, 127, 128] {
        let mask = if n == 128 {
            u128::MAX
        } else {
            (1u128 << n) - 1
        };
        let a = 0xa5a5_0123_fedc_4567_89ab_cdef_1357_2468u128 & mask;
        let b = 0x3141_5926_5358_9793_2384_6264_3383_2795u128 & mask;
        let bits = |x: u128| {
            if n == 0 {
                String::new()
            } else {
                format!("{x:0n$b}")
            }
        };
        source.push_str(&format!("{{let a:=BitVec(\"{}\");let b:=BitVec(\"{}\");wt Str(a&b),Str(a|b),Str(a^b),Str(~a),a.count();",bits(a),bits(b)));
        expected.push_str(&format!(
            "{} {} {} {} {}\n",
            bits(a & b),
            bits(a | b),
            bits(a ^ b),
            bits(!a & mask),
            a.count_ones()
        ));
        for shift in [-129i64, -65, -64, -1, 0, 1, 63, 64, 65, 129] {
            let left = if shift.unsigned_abs() >= 128 {
                0
            } else if shift < 0 {
                a >> (-shift as u32)
            } else {
                (a << (shift as u32)) & mask
            };
            let right = if shift.unsigned_abs() >= 128 {
                0
            } else if shift < 0 {
                (a << (-shift as u32)) & mask
            } else {
                a >> (shift as u32)
            };
            source.push_str(&format!("wt Str(a<<({shift})),Str(a>>({shift}));"));
            expected.push_str(&format!("{} {}\n", bits(left), bits(right)));
        }
        source.push('}');
    }
    let out = Case::new(&source).run("");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, expected.as_bytes());
}

#[test]
fn bigint_and_packed_integer_input_use_standard_io() {
    let source = "in a:BigInt;in b(3):IntArray;wt a,b;wt a+BigInt(b.sum());";
    let case = Case::new(source);
    for options in [vec![], vec!["--no-opt"]] {
        let out = case.run_with_options("-123456789012345678901234567890 4 -5 6", &options);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.stdout,
            b"-123456789012345678901234567890 4 -5 6\n-123456789012345678901234567885\n"
        );
    }
}
