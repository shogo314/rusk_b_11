# 読み取り専用のアクセス

`readonly` は、その名前やそこから共有して得た値を通じた変更を禁止します。省略時は `mutable` です。`mutable` を明示しても、引き継いだ読み取り専用の制約を解除することはできません。

```text
let original := 1;
readonly let view := original;
original = 2;
wt view;                       // 2：元の共有先の変更は見える
// view = 3;                   // エラー
// let alias := view; alias += 1; // 別名を通じた変更もエラー
let detached = view;
detached += 1;                 // 独立したコピーは変更できる
```

## 宣言と引数

`let` の前、名前付き関数・型引数付き関数・無名関数の引数名の前に指定します。分解宣言では各変数に適用します。

```text
readonly let (left, right) := (1, 2);
fn twice(readonly value: Int) -> Int { return value * 2; }
let next := (readonly value: Int) => value + 1;
mutable let working := 3;
```

[fixed / rebindable](bindings.md) と組み合わせられ、順番は自由です。`readonly fixed let x := 1;` と `fixed readonly let x := 1;` は同じ制限になります。

`readonly` 自体は名前の結び直しを禁止しません。ただし結び直した後も、その宣言の読み取り専用という制限は維持します。

```text
readonly let value := 1;
value := 2;                    // 許可
// value = 3;                  // エラー
```

読み取り専用でない別名に `:=` で新しい値を結び直した場合は、以前の値から伝わった制約を引き継ぎません。未初期化の宣言にも指定でき、通常の初期化前使用検査に従います。

同じ種類の注釈の重複や `readonly mutable` の併記はエラーです。注釈だけでオーバーロードは区別できません。この束縛注釈は構造体フィールド、`in`・`for`・パターンの束縛には指定できません。フィールドの型には`readonly T`を指定できます。外部関数の引数では、[externの作成者が保証する契約](../development/extern.md#extern-parameter-contracts)として指定できます。

## 要素・フィールド・呼び出し

読み取り専用の配列・文字列の要素、構造体・タプルのフィールド、enumのpayloadにも制約を伝えます。`=`、複合代入、要素・フィールドへの `:=`、配列の追加などを検査します。

```text
readonly let values := [1, 2];
wt values.len(), values[0];
// values[0] = 3;              // エラー
// values.push_back(3);        // エラー
```

Rusk関数では、呼び出し先に `readonly` がなくても、本体の効果を解析して検査します。引数から得た別名の更新も含みます。通常のRusk関数、型引数付き関数、無名関数・コールバックの呼び出しを扱います。externに `readonly` を指定すると、呼び出し時にその引数経由で値や要素を変更しないという作成者の保証を使います。

```text
fn read(value: Int) -> Int { return value + 1; }
fn bump(value: Int) { value += 1; }
readonly let x := 1;
wt read(x);
// bump(x);                    // エラー：引数1を変更する
```

戻り値が引数を共有する場合、呼び出し元へも制約を伝えます。関数内で明示的に読み取り専用にした値を返す場合も、その制約を維持します。

```text
fn view(readonly value: Int) -> Int { return value; }
let result := view(1);
// result = 2;                 // エラー
let detached = result;
detached = 2;
```

無名関数の捕捉、捕捉した値の返却、配列や構造体などへの格納によっても、制約を変更可能なアクセスへ変換することはできません。別の共有先を凍結する機能ではないため、値が不変であるという最適化の根拠には単独では使いません。

## 現在の保守的な検査と制限

契約検査は保守的です。分岐・ループ・再帰を含む制御フローで、到達する可能性のある読み取り専用の制約を合流させます。一般の条件の真偽や、すべての共有先の独立性を証明するものではありません。

ローカルに直接構築した既知のオブジェクトは、別名・入れ子・返却した共有先まで追跡します。フィールドへの格納だけで無関係な別のオブジェクトや他のフィールドを読み取り専用にはしません。ただし格納内容は弱い更新で合流させるため、同じフィールドを置き換えた後も以前の制約が残る場合があります。引数由来の格納先、外へ保持されたオブジェクト、未知の返却や配列の操作には引き続き広く保守的な制約を適用し、無関係な操作も拒否することがあります。スコープ終了だけではこの不明性を解除しません。

契約がない外部実装や、子の操作・共有先の効果を証明できない走査・コンテナ操作は不透明です。読み取り専用の値を渡すと、実際には読み取りだけでも拒否する場合があります。配列のコピー・出力などの契約は、通常の型付き参照で選んだ要素操作の効果を検査してから有効にします。レコードや外部型を含む配列では、利用者定義のコピー関数などを呼ぶ可能性があるため、要素型の見た目だけでは信頼しません。

同じ解析を[noescape / escape](escape.md)の検査にも使います。解析結果を[引数借用と値保持](../development/performance.md)へ利用しますが、readonlyの指定だけを最適化の根拠にはしません。ヒープ共有や各操作の証明精度は継続して改善する対象です。

## エラー表示とIR

検出した変更には `cannot modify readonly access`、外部実装や共有先などの不明性で安全を証明できない場合には `cannot prove readonly access is preserved` を表示します。操作の位置、元の変数名・宣言位置、`readonly` の位置を示し、呼び出しでは関数名と引数番号も示します。フィールドなどに含まれた制約はIRの射影経路も表示します。`checked access`には、実際に検査した操作対象の束縛名と位置を表示します。一時的な共有参照やフィールド・payloadの取得を経由する場合は、その取得先も示します。getterなどの呼び出し結果を直接更新する場合は、関数名と呼び出し位置を示します。同名の変数は宣言IDと位置で区別し、結び直し前の代入履歴を推測して原因として表示しません。

`readonly propagation path`には、起点から別名・代入・格納・フィールド・payload・捕捉を通じた伝播の候補を示します。通常の関数や静的な関数値を経由する場合は、仮引数・局所別名・返却、変更箇所まで示します。結び直しで制約がなくなった値の古い履歴は使いません。

例えば次の呼び出しは拒否されます。

```rusk
fn change(x: Int) { let target := x; target = 2; }
readonly let original := 1;
change(original);
```

経路部分には、位置情報とともに次の段階が表示されます。

```text
readonly binding 'original' introduced here
readonly argument enters parameter 1 of 'change'
binding 'x' declared here
binding 'target' declared here
argument 1 of '__assign__' may modify this access
```

不明な外部効果や未追跡のヒープ共有などで経路を精密に示せない場合は、`uncertain readonly propagation path`と理由を表示します。readonlyの制約が集約の内側だけにあり、呼び出し先の効果からどのメンバーを変更するか分からない場合も、安全を証明できないという表示になります。

この経路は静的解析上の候補で、実行済みの操作列ではありません。表示する段階と呼び出し先の探索は各16段までで、長い経路や再帰では省略を明示します。診断用の追加解析にも上限があり、経路を再構成できなくても元の拒否判定と起点の表示は維持します。

ASTでは明示した注釈、HIR/LIRでは `AnnotateAccess` と関数の `access` 要約を確認できます。要約には戻り値・外部への保持に伝わる制約を記録し、注釈の由来と関数の効果解析を分離しています。HIR/LIR・C++を出力する前に検査します。注釈は実行時の処理やC++の `const` を追加しません。


## 型で表すreadonly {#readonly-types}

`readonly T`は、Tを共有して読むアクセスを表します。返却型、フィールド型、型引数に使えます。`Vec<readonly Int>`では外側の配列を変更でき、取得したIntは変更できません。`readonly Vec<Int>`では、配列の構造変更も要素を通した更新も禁止します。

```text
fn view(value: Int) -> readonly Int { return value; }
let original := 1;
let items: Vec<readonly Int> := [view(original), view(2)];
items.push_back(view(3));
original = 4;
wt items;                      // 4 2 3
let detached = items[0];
detached += 1;
wt detached;                   // 5
// items[0] = 9;              // 要素自体の更新はエラー
```

`items[0] := view(7)`のような格納先の結び直し、末尾への追加・取り出し、clearは外側の変更です。外側がmutableなら使えます。`mutable`を明示しても、要素のreadonlyは解除しません。別のmutableな共有経路から元の値を変更すると、readonlyの経路からも変更が見えます。

同じ配列を共有したまま`Vec<T>`と`Vec<readonly T>`を相互変換することはできません。ジェネリックを経由しても同じ規則です。要素の権限を変えて集める場合は、新しい外側のコンテナを構築します。`Vec<readonly Vec<Int>>`のような入れ子や、struct・enumの格納型にも指定できます。

根元の`readonly`は通常の読み取り操作や型推論で基の型へ接続しますが、そのアクセス制限は解析に残ります。`fn f(x: Int)`と`fn f(x: readonly Int)`の違いだけではオーバーロードを選べません。`Holder<Int>`と`Holder<readonly Int>`は異なる具体化です。`readonly readonly T`は`readonly T`と同じです。Unitにも指定でき、基の型のUnit ABIを保ちます。

コピーが独立した値を返すと確認できる場合、根元のreadonlyを引き継がずに変更可能なコピーを作れます。一方、`Vec<readonly T>`をコピーしても要素型の指定は残ります。各要素を読み出して独立にコピーすれば、別のmutableな要素を作れます。共有を返す手動`__copy__`の制限は解除しません。

externの署名に明示したreadonly引数型は、[readonly引数の契約](../development/extern.md#extern-parameter-contracts)と同じ作成者の保証を伴います。単にジェネリックのTをreadonly型で具体化したことだけでは、未知のC++本体へ非変更の保証を与えません。readonly返却型は、呼び出し元にも制限を付けます。readonlyな内容を持つ値から未知のexternが返す共有先や保持先も、権限を失わないように保守的に扱います。

既知のRusk関数の書き込み先は、フィールドなどの経路を使って区別します。外側だけの変更と証明できる標準操作も、格納したreadonlyの値を理由に拒否しません。例えば`Vec<Option<readonly Int>>`への追加では、payloadのreadonlyを保ったまま外側を変更できます。辞書・集合の操作の多くはコールバック・保持・解放の契約が不透明なため、readonlyを含む型で拒否される場合があります。変更・保持・解放の影響を証明できない外部操作や、解析の深さ・追跡量の上限を超える場合は、引き続き安全側に拒否します。
