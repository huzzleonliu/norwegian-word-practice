# Instructions

## Local Lexicon Editor

Customize a private lexicon in the browser: single-entry add, dictionary/AI-assisted fill-in, bulk preview write, plus in-table editing and export.

### Access Requirements

This page is gated by NFT ownership:

* If no wallet is connected, the app triggers **Connect Wallet**
* If a wallet is connected but does not hold an NFT from this collection, **Mint NFT** opens in a new tab
* You can use this page only after connecting a wallet that holds at least one NFT from the series

Open it from **Local Lexicon Editor** on the practice mode selection page (bottom-right), or go to `/editor` (the same gate applies).

### Page Overview

1. **Status console** — operation feedback; reminds you to **Confirm Changes** before exporting  
2. **Single Add** — manually add one entry  
3. **Dictionary-first lookup** — Ordbok → Gemini helpers; results go to the single form or bulk preview  
4. **Bulk Add** — write preview rows into the lexicon  
5. **Lexicon Browser (Edit)** — edit/delete/search in a table; export CSV at the bottom  

The lexicon is stored in the browser (usually restored after refresh). Use the top-left control to return to practice mode selection. This page does not start practice directly.

### Single Add

Fill Norwegian base form and inflection fields by part of speech, plus translations and tags, then click **Add Entry**.

* **Clear** resets the single-entry form (usually without clearing saved tokens / default tags)
* Successful adds are written into the current lexicon immediately

### Dictionary-first Lookup (Ordbok → Gemini)

Auto-fill fields from a Norwegian word form:

1. Optionally configure Gemini / Google Translate tokens and test connectivity  
2. Enter the word form; you may set default tags  
3. Run lookup: Ordbok first, Gemini for gaps when needed; Google Translate often fills Chinese/English  
4. **Ordbok only** disables the Gemini fallback  

Results may:

* fill the **Single Add** form above, or  
* go to the **Bulk Add** preview table (multiple senses / parts of speech)

### Bulk Add

Shows multi-entry previews from lookup. Click **Add Entries** to write them into the lexicon, remove unwanted preview rows, or **Clear** the preview area.

### Lexicon Browser (Edit)

View and edit the current lexicon in a spreadsheet-like table:

* Edit cells; mark rows for deletion (drag to multi-select)
* You **must click Confirm Changes** before deletions/edits are written and id checks run
* Search, column visibility, sorting, column resize, and pagination are available

### Export Encrypted Lexicon (.nwpdict)

The export control sits at the bottom of the lexicon browser, **to the left of Confirm Changes**. There is no import button on this page anymore—import on the practice mode selection page.

* Export generates an encrypted lexicon file (`.nwpdict`), not plain CSV
* Export uses the **committed** lexicon, not unconfirmed table drafts
* After edits or deletions, click **Confirm Changes**, then **Export Lexicon File**

You can import the file directly via **Import Lexicon File** on the practice mode selection page (it will auto-decrypt). Plain CSV import remains supported.
