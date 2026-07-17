# Instructions

## Local Lexicon Editor

Edit a local lexicon in the browser: manual add, dictionary/AI-assisted fill-in, table editing, and CSV import/export. The console at the top shows status. After editing the table, click **Confirm Changes** before exporting.

### Single Add

Fill in Norwegian base form and inflection fields by part of speech, plus translations and tags, then click **Add Entry**.

* **Clear** resets the single-entry form (usually without clearing saved tokens / default tags)
* Successful adds are written into the current lexicon immediately

### Dictionary-first Lookup (Ordbok → Gemini)

Auto-fill fields from a Norwegian word form:

1. Optionally configure Gemini / Google Translate tokens and test connectivity
2. Enter the word form to look up; you may also set default tags
3. Run lookup: Ordbok first, Gemini for missing fields when needed; Google Translate often fills Chinese/English
4. **Ordbok only** disables the Gemini fallback

Results may:

* fill the **Single Add** form above, or
* go to the **Bulk Add** preview table (multiple senses / parts of speech)

### Bulk Add

Shows multi-entry previews from lookup. Click **Add Entries** to write them into the lexicon, remove unwanted preview rows, or **Clear** the preview area.

### Lexicon Browser (Edit)

View and edit the current lexicon in a spreadsheet-like table:

* Edit cell values
* Mark rows for deletion (drag to multi-select)
* Click **Confirm Changes** to apply deletions, save edits, and run id checks
* Search, column visibility, sorting, and pagination work similarly to the word-selection page

### Import and Export

* **Import Lexicon CSV**: import from your computer (usually overwrites the current lexicon)
* **Export Lexicon CSV**: download the current lexicon. Confirm table edits first

Return to the practice mode selection page from the top left. This page does not start practice directly—go back, load/import the lexicon, then practice.
