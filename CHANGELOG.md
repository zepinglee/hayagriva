# 0.4.0

## Breaking changes:

Hayagriva now uses the [Citation Style Language](https://citationstyles.org) to
encode formatting styles. This means that Hayagriva's own formatting styles have
been deprecated.

### For users:
- The YAML input format has changed.
    - Titles and formattable strings have been merged into one type. All
      formattable strings can have a shorthand now.
    - Formattable Strings do not have `title-case` and `sentence-case` keys
      anymore. `shorthand` has been renamed to `short`. To prevent changes of
      the text case of formattable strings, you can use braces. Enclose a part
      of a formattable string (or `short`) in `{braces}` to print it as-is.
    - The fields `doi`, `isbn`, and `issn` have been moved to `serial-number`
      which can now be a dictionary containing these and arbitrary other serial
      numbers like a `pmid` (PubMed ID) and `arxiv` (ArXiv Identifier).
    - The `tweet` entry type has been renamed to `post`.
    - All numeric variables can now also contains strings. Numbers can have
      string affixes.

Refer to the updated
[file format](https://github.com/typst/hayagriva/blob/main/docs/file-format.md)
docs for examples.

### For developers:
- To use a CSL style, you can either supply a CSL file or use an archive of
  provided styles with the `archive` feature.
- The `from_yaml_str` function will now return the new `Library` struct, with the
  entries within.
- The `Database` struct has been replaced by the easier to handle
  `BibliographyDriver`.
- We switched from `yaml_rust` to `serde_yaml`. The `Entry` now implements
  `serde`'s `Serialize` and `Deserialize` traits. Hence, the `from_yaml` and
  `to_yaml` functions have been deleted.
- Brackets are no longer individually overridable. Instead, use the new
  `CitePurpose`.
- `Entry::kind` has been renamed to `Entry::entry_type`.
    - The citation styles `AuthorTitle` and `Keys` have been removed but can be
      realized with CSL.

This release fixes many bugs and makes Hayagriva a serious contender for
reference management.

## Other changes

- We added the entry types `Performance` and `Original`.
- We added the field `call-number`.


# 0.3.2

Fixes a title case formatting bug introduced in the previous release.

# 0.3.1

_Bug Fixes:_
- Added an option to turn off abbreviation of journals (thanks to @CMDJojo)
- Fixed bugs with title case formatting (thanks to @jmskov)
- Fixed off-by-one error with dates in APA style (thanks to @bluebear94)
- Fixed supplements in the Alphanumeric and AuthorTitle styles (thanks to @lynn)
- Fixed bugs with sentence case formatting
- Fixed `verbatim` option
- Fixed terminal formatting
- Fixed some typos (thanks to @kianmeng and @bluebear94)

# 0.3.0

*Breaking:*
- Updated to `biblatex` 0.8.0

*Bug Fixes:*
- Fixed string indexing for titles, removed panic
- More permissive BibLaTeX parsing

# 0.2.1

*Bug Fixes:*
- Fixed APA bibliography ordering

# 0.2.0

*Breaking:*
- Replaced `NoHyphenation` formatting with `Link` formatting
- Switched to newest BibLaTeX (which is part of the public API)

*Bug Fixes:*
- Fixed IEEE bibliography ordering
- Fixed A, B, C, ... suffixes for Author Date citations
- Removed `println` calls

# 0.1.1

🐞 This release fixes the documentation of the CLI in the `README.md` file.
✨ There are new options for bracketed citations in the CLI.
✅ No breaking changes.

# 0.1.0

🎉 This is the initial release!