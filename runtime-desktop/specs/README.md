# Specs

Spesifikasi teknis library runtime-desktop.

Catatan baseline engine:

- Backend desktop membaca style dari IR canonical melalui `effective_style_decls(...)`.
- Phase sizing terbaru menambahkan dukungan `flex`, `flex-grow`, `flex-shrink`, dan `flex-basis` untuk parity web-desktop.
- Scaffold desktop menghasilkan `native-app/src/actions.rs` untuk registry action + helper expression set.
- Expression `action set` kompleks dievaluasi lewat RPN (`eval_set_expression_rpn`) agar urutan precedence/kurung tetap selaras dengan runtime web.
