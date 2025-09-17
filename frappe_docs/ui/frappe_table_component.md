# Frappe “Table” Component on a Custom Page

Use Frappe’s **Table** fieldtype (the same grid used in DocTypes) inside a **custom page** to get add/remove rows, column types, and validation UI—without opening a form.

This guide shows how to render it, feed data, read data back, and save it yourself.

---

## What the “Table” Component Is (and Isn’t)

- **Is:** the same grid UI used by a DocType’s child table field.
- **Needs:** a **Child DocType** (its schema defines columns).
- **Doesn’t do automatically on a custom page:** load/save to the database. You control data source and persistence.

If you don’t want to define a Child DocType, use **frappe-datatable** instead (see the end).

---

## Prerequisites

1. **Child DocType**

   - Create a Child DocType (e.g. `Invoice Item`) with `Is Child Table = 1`.
   - Add fields (e.g. `item_code` (Link), `qty` (Float), `rate` (Currency), etc.).

2. **A Custom Page** scaffolded in your app.

---

## Quick Start (Minimal)

```js
// file: your_app/your_app/page/my_page/my_page.js
frappe.pages["my-page"].on_page_load = function (wrapper) {
  const page = frappe.ui.make_app_page({
    parent: wrapper,
    title: "My Table on a Custom Page",
    single_column: true,
  });

  // Container for the table
  const $holder = $('<div id="my-table" />').appendTo(page.body);

  // 1) Create a Table control (Grid) using a Child DocType schema
  frappe.ui.form
    .make_control({
      parent: $holder,
      df: {
        fieldtype: "Table",
        label: "Items",
        options: "Invoice Item", // <-- your Child DocType name
      },
      render_input: true,
    })
    .then((control) => {
      // 2) Preload data (optional)
      control.df.data = [
        { item_code: "SKU-001", qty: 2, rate: 75000 },
        { item_code: "SKU-002", qty: 1, rate: 125000 },
      ];
      control.refresh(); // renders grid rows

      // 3) Read data back (e.g., to save)
      page.set_primary_action("Save", async () => {
        const rows = control.grid.get_data(); // array of row objects
        // Do something with rows (call a whitelisted method, etc.)
        await frappe.call("your_app.your_app.page.my_page.my_page.save_rows", {
          rows,
        });
        frappe.show_alert({ message: __("Saved!"), indicator: "green" });
      });
    });
};
```

---

## Rendering Details

- Use `frappe.ui.form.make_control({ df: { fieldtype: "Table", options: "<Child DocType>" } })`.
- `options` **must** be your **Child DocType** name (not a DocField label).
- Call `control.refresh()` after you set or mutate `control.df.data`.

---

## Loading Data

You have two common patterns:

### A) Feed raw objects directly

```js
control.df.data = existing_rows; // array of objects matching child fields
control.refresh();
```

### B) Build rows programmatically

```js
// Adds a single empty row:
control.grid.add_new_row();

// Or add many:
for (const r of rows) control.grid.add_new_row(r);
control.grid.refresh();
```

> Tip: Rows should use **fieldnames** from the Child DocType (e.g., `qty`, `item_code`, `rate`).

---

## Reading Data

```js
const rows = control.grid.get_data();
// rows = [{ item_code: 'SKU-001', qty: 2, rate: 75000, ... }, ...]
```

This returns “clean” JSON of the grid values (ignores internal grid metadata).

---

## Common Grid Operations

```js
// Add blank row
control.grid.add_new_row();

// Remove selected rows
const selected = control.grid.get_selected();
control.grid.remove_rows(selected);

// Refresh UI after programmatic changes
control.grid.refresh();

// Access row objects (advanced)
for (const row of control.grid.grid_rows || []) {
  // row.doc is the underlying row data
  // row.remove() to remove this row
}
```

---

## Validation

You have two layers:

1. **Child DocType field rules** (Reqd, Options, Types) → enforced by the grid UI.
2. **Custom checks** (before saving):

   ```js
   const rows = control.grid.get_data();

   // Example: qty must be > 0
   const bad = rows.find((r) => !r.qty || r.qty <= 0);
   if (bad) {
     frappe.msgprint(__("Qty must be greater than zero"));
     return;
   }
   ```

---

## Saving (Server Round-Trip)

Create a whitelisted method (Python) to persist wherever you want (a parent DocType, a custom table, or your own logic).

```python
# file: your_app/your_app/page/my_page/my_page.py
import frappe

@frappe.whitelist()
def save_rows(rows: list[dict] | None = None):
    """Persist the grid rows somewhere (example only)."""
    rows = rows or []
    # Example: write into a parent DocType "Invoice Draft" + its child "Invoice Item"
    parent = frappe.new_doc("Invoice Draft")
    for r in rows:
        parent.append("items", r)  # "items" = child table fieldname on the parent
    parent.insert(ignore_permissions=True)
    frappe.db.commit()
    return {"name": parent.name, "count": len(rows)}
```

> Replace with your own persistence rules. On a **custom page**, you decide the storage model.

---

## Events & Hooks You Can Use

While there’s no formal “grid event bus” API, practical hooks include:

```js
// Listen to changes within the grid UI
control.grid.wrapper.on("change", "input, select", (e) => {
  // e.target has the edited input; you can re-sum totals, etc.
});

// Recalculate totals on refresh
const recompute = () => {
  const rows = control.grid.get_data();
  const total = rows.reduce((s, r) => s + (r.qty || 0) * (r.rate || 0), 0);
  $("#grand-total").text(frappe.format(total, { fieldtype: "Currency" }));
};

control.grid.refresh = ((orig) =>
  function () {
    const ret = orig.apply(this, arguments);
    recompute();
    return ret;
  })(control.grid.refresh);
```

---

## Read-Only / Disabled Mode

```js
// Disable add/remove and editing:
control.grid.only_sortable(); // disables editing cells (lightweight)
control.grid.wrapper.addClass("disabled-grid"); // CSS approach
// Or, for stricter control, set fields as Read Only in Child DocType and hide add/remove buttons:
control.grid.wrapper.find(".grid-add-row, .grid-remove-rows").hide();
```

(For robust locking, enforce on the server as well.)

---

## Formatting & Link Fields

- The grid uses standard Frappe formatters (Currency, Int, Float, Link, etc.).
- For `Link` fields, typeahead works if the linked DocType is accessible and has a search field.
- To display formatted values externally:

  ```js
  const currency = frappe
    .get_meta("Invoice Item")
    .fields.find((f) => f.fieldname === "rate");
  const formatted = frappe.format(125000, currency, {}, "Currency");
  ```

---

## Permissions

The grid won’t check a parent form’s permissions on a custom page. Enforce permissions in your whitelisted methods (e.g., role checks) before writing to DB.

---

## Performance Notes

- Large datasets: prefer **server-side pagination** or load a subset, not thousands of rows at once.
- Avoid frequent `control.refresh()` calls—batch updates then refresh once.
- If you need virtualization and big-data scrolling, consider **frappe-datatable**.

---

## Troubleshooting

- **Blank grid**: ensure `df.options` is exactly your **Child DocType** name and that DocType exists.
- **Columns missing**: confirm fields are on the Child DocType (not `Hidden`) and you don’t override `in_list_view` unexpectedly.
- **No save**: remember this is not a Form; implement your own save routine.
- **Link field not searching**: check the linked DocType’s `search_fields` and user permissions.

---

## When to Use `frappe-datatable` Instead

Use **frappe-datatable** when:

- You don’t want to create a Child DocType.
- You need huge datasets with virtualization.
- You want pure display or a custom edit model.

Minimal example:

```js
// yarn add frappe-datatable (in app build step) or use the included asset if available
const dt = new DataTable("#table", {
  columns: [
    { name: "Item Code", id: "item_code", editable: true },
    { name: "Qty", id: "qty", editable: true, format: (v) => +v },
    { name: "Rate", id: "rate", editable: true },
  ],
  data: [
    { item_code: "SKU-001", qty: 2, rate: 75000 },
    { item_code: "SKU-002", qty: 1, rate: 125000 },
  ],
});

const rows = dt.getData(); // read back
```

---

## Complete Example (Custom Page with Save)

**JS (page front-end)**

```js
frappe.pages["items-planner"].on_page_load = function (wrapper) {
  const page = frappe.ui.make_app_page({
    parent: wrapper,
    title: "Items Planner",
    single_column: true,
  });

  const $actions = $(`
    <div class="flex items-center gap-2 mb-3">
      <button class="btn btn-sm btn-primary" id="add-row">Add Row</button>
      <button class="btn btn-sm btn-secondary" id="load">Load Sample</button>
      <button class="btn btn-sm btn-primary" id="save">Save</button>
      <div class="ml-auto text-muted">Grand Total: <b id="grand-total">0</b></div>
    </div>
  `).appendTo(page.body);

  const $holder = $('<div id="grid-holder" />').appendTo(page.body);

  let gridControl;

  frappe.ui.form
    .make_control({
      parent: $holder,
      df: { fieldtype: "Table", label: "Plan Items", options: "Invoice Item" },
      render_input: true,
    })
    .then((control) => {
      gridControl = control;

      const recompute = () => {
        const rows = control.grid.get_data();
        const total = rows.reduce(
          (s, r) => s + (r.qty || 0) * (r.rate || 0),
          0,
        );
        $("#grand-total").text(frappe.format(total, { fieldtype: "Currency" }));
      };

      control.grid.wrapper.on("change", "input, select", recompute);

      $("#add-row").on("click", () => {
        control.grid.add_new_row();
        control.grid.refresh();
      });

      $("#load").on("click", () => {
        control.df.data = [
          { item_code: "SKU-001", qty: 3, rate: 50000 },
          { item_code: "SKU-ABC", qty: 1, rate: 250000 },
        ];
        control.refresh();
      });

      $("#save").on("click", async () => {
        const rows = control.grid.get_data();
        if (rows.length === 0) return frappe.msgprint(__("No rows to save"));

        // Simple client validation
        if (rows.some((r) => !r.item_code || !r.qty || r.qty <= 0)) {
          return frappe.msgprint(
            __("Please fill Item Code and positive Qty for all rows."),
          );
        }

        const { message } = await frappe.call(
          "your_app.your_app.page.items_planner.items_planner.save_rows",
          { rows },
        );
        frappe.show_alert({
          message: __("Saved: {0}", [message.name]),
          indicator: "green",
        });
      });
    });
};
```

**Python (server save)**

```python
import frappe

@frappe.whitelist()
def save_rows(rows: list[dict] | None = None):
    """Example: persist rows into a parent 'Plan' with child 'Invoice Item'."""
    rows = rows or []
    if not frappe.has_permission('Plan', 'create'):
        frappe.throw('Not permitted')

    plan = frappe.new_doc('Plan')
    for r in rows:
        plan.append('items', r)  # 'items' is the child table field on Plan
    plan.insert()
    frappe.db.commit()
    return {"name": plan.name, "count": len(rows)}
```

> Replace `'Plan'`/`'items'` with your actual parent DocType and child fieldname, or write custom storage logic.

---

## FAQ

**Q: Do I have to use a Child DocType?**
**A:** Yes, for the **Table** fieldtype. Otherwise use **frappe-datatable**.

**Q: Can I reuse the same Table across pages?**
**A:** Yes. Create a helper that builds the control and injects it wherever needed.

**Q: Can I use grid filters, totals, etc.?**
**A:** Totals are DIY (as shown). For filtering, either transform `get_data()` or prefer **frappe-datatable**.

---

## TL;DR

- Use `frappe.ui.form.make_control` with `fieldtype: "Table"` and `options: "<Child DocType>"`.
- Feed data via `control.df.data = [...]` + `control.refresh()`.
- Read with `control.grid.get_data()`.
- Implement your own **save** (whitelisted method).
- For big/standalone tables without a Child DocType, pick **frappe-datatable**.
