# Specs

Spesifikasi teknis library runtime-web.

Catatan baseline engine:

- Backend web membaca style dari IR canonical melalui `effective_style_decls(...)`.
- Kontrak ini menjaga semantic style tetap sama dengan backend desktop.
- Runtime web memprioritaskan handler FL generated (`formoGeneratedActions`) sebelum fallback ke `window.formoActions`.
- Expression `action set` kompleks dievaluasi dengan RPN (`evalSetExpressionRpn`) untuk menjaga urutan precedence/kurung setara desktop.
