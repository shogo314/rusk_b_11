# β02のライブラリ利用例

旧`library/`の実装はβ11で標準ライブラリの[`lib/data_structures/`](../../lib/data_structures/)へ移しました。includeは不要です。

- `UnionFind`：`unionfind.rusk`
- `FenwickTree`：`fenwick-tree.rusk`。更新は`apply`、集計は`prefix_prod`・`prod`
- `SumSegTree`：`sum-segtree.rusk`

利用例は[Unionfind](../library_checker/unionfind.test.rusk)、[Fenwick木](../library_checker/point_add_range_sum.test.rusk)、[セグメント木](../library_checker/point_add_range_sum_segment_tree.test.rusk)にあります。共有・コピーと必要な演算は[データ構造の説明](../../docs/library/data-structures.md)を参照してください。
