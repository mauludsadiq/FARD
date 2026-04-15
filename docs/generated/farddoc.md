# farddoc

farddoc — Generate Markdown documentation from FARD source comments.

Doc comment syntax:
/// Single line doc comment
fn name(params) { ... }

Usage:
fardrun run --program apps/farddoc.fard -- --program file.fard
fardrun run --program apps/farddoc.fard -- --package path/to/pkg

*Source: `apps/farddoc.fard`*

---

## `get_arg(args, flag)`

*Line 21*

## `collect_fard_files(path)`

*Line 36*

## `list_last(xs)`

*Line 56*

## `list_prepend(item, xs)`

*Line 60*

## `list_filter_map(xs, f)`

*Line 64*

## `str_trim_prefix(s, prefix)`

*Line 68*

## `extract_module_name(filename)`

*Line 75*

## `parse_one_param(p)`

*Line 84*

## `parse_params(param_str)`

*Line 94*

## `extract_items(source, filename)`

*Line 101*

## `render_param_sig(p)`

*Line 157*

## `render_param_doc(p)`

*Line 162*

## `render_item(item)`

*Line 169*

## `render_markdown(items)`

*Line 200*

## `process_file(filepath)`

*Line 206*

