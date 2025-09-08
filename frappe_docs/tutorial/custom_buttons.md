# Creating Custom Buttons (How-To & Examples)

This guide shows **four** reliable ways to add buttons in Frappe/ERPNext, with production-ready snippets.

---

## 1) Form Toolbar Button (Client Script)

### When to use

- You want a button in the **form toolbar** (top-right) of a DocType form.
- Best for one-off actions on the current document.

### Steps

1. Go to: **Settings → Customization → Client Script → New**
2. Fill:

   - **Script Type**: _Client_
   - **Referenced Doctype**: e.g., `Sales Order`

3. Paste script below and Save.

### Example

```javascript
// Client Script for "Sales Order"
frappe.ui.form.on("Sales Order", {
  refresh(frm) {
    // Only show when the document already exists
    if (frm.is_new()) return;

    // Add a single button
    const btn = frm.add_custom_button(
      __("Send to External System"),
      async () => {
        await frappe.confirm(__("Are you sure you want to push this order?"));

        // Call a server method (see Section 4)
        frappe.call({
          method:
            "my_app.my_app.doctype.sales_order.sales_order.push_to_external",
          args: { name: frm.doc.name },
          freeze: true, // shows spinner
          freeze_message: __("Pushing..."),
          callback(r) {
            if (!r.exc) {
              frappe.msgprint(__("Pushed successfully."));
              frm.reload_doc();
            }
          },
        });
      },
    );

    // Make it stand out
    btn.addClass("btn-primary");

    // Optional: group into a dropdown menu
    frm.add_custom_button(
      __("Print Picking List"),
      () => {
        frappe.set_route("query-report", "Picking List", {
          sales_order: frm.doc.name,
        });
      },
      __("Actions"),
    );

    frm.page.set_inner_btn_group_as_primary(__("Actions")); // makes the group primary
  },
});
```

**Notes**

- `frm.add_custom_button(label, handler, group?)`
- Use `__()` for translatable labels.
- Guard with `if (frm.is_new()) return;` to avoid showing on unsaved docs.

---

## 2) Button Field inside the Form (DocType “Button” field)

### When to use

- You want a button **inside the form body**, near related fields.
- Easy to configure via **Customize Form**.

### Steps

1. **Customize Form** → choose your DocType.
2. Add a new field:

   - **Label**: `Recalculate Totals`
   - **Field Type**: `Button`
   - **Fieldname**: `recalculate_totals`

3. Add this Client Script:

```javascript
frappe.ui.form.on("Sales Order", {
  // Event name is the button's fieldname
  recalculate_totals(frm) {
    // Client-only logic or a server call
    frappe.call({
      method: "my_app.utils.recalc_sales_order",
      args: { name: frm.doc.name },
      freeze: true,
      callback: () => {
        frappe.show_alert({ message: __("Recalculated"), indicator: "green" });
        frm.reload_doc();
      },
    });
  },
});
```

**Notes**

- Clicking a **Button** field triggers an event named exactly as its **fieldname**.

---

## 3) List View Buttons (Bulk Actions)

### When to use

- You need actions from the **List View** (e.g., bulk close, export, reprocess).
- Good for **multi-select** operations.

### Option A — Client Script (List View)

> Use **Client Script** with **View = List** (Frappe v13+).

```javascript
// Client Script for List View of "ToDo"
frappe.listview_settings["ToDo"] = {
  onload(listview) {
    listview.page.add_inner_button(__("Mark Selected as Done"), async () => {
      const names = listview.get_checked_items().map((d) => d.name);
      if (!names.length) {
        frappe.msgprint(__("Please select at least one row."));
        return;
      }
      await frappe.confirm(__("Mark {0} items as Done?", [names.length]));

      frappe.call({
        method: "my_app.todo.bulk_mark_done",
        args: { names },
        freeze: true,
        callback: () => {
          frappe.show_alert({ message: __("Updated"), indicator: "green" });
          listview.refresh();
        },
      });
    });
  },
};
```

### Option B — App code (list view settings file)

> In a custom app, create `public/js/<doctype>_list.js` and include it via hooks if you prefer code-managed assets.

---

## 4) Server Method (Whitelisted) for Buttons

### When to use

- Your button must perform **write** operations, integrate with external APIs, or enforce **permission checks** securely.

### Example (Python)

```python
# my_app/my_app/doctype/sales_order/sales_order.py
import frappe

@frappe.whitelist()
def push_to_external(name: str):
    # Check perms explicitly — don't trust the client
    if not frappe.has_permission('Sales Order', 'write', name=name):
        frappe.throw('Not permitted', frappe.PermissionError)

    doc = frappe.get_doc('Sales Order', name)

    # --- Your business logic ---
    # e.g., call external API, update doc status, log, etc.
    # external_api.push(doc.as_dict())

    # Example mutation:
    doc.db_set('custom_pushed_flag', 1)

    # Optional: add a comment/log
    doc.add_comment('Comment', text='Pushed to external system.')
    return {'ok': True}
```

**Security Tips**

- Always mark callable functions with `@frappe.whitelist()`.
- Re-check permissions server-side with `frappe.has_permission(...)`.
- Avoid trusting client-supplied data; load docs via `frappe.get_doc`.

---

## 5) Page/Workspace Buttons (Non-DocType pages)

### When to use

- You build a **custom desk page** and want **primary/secondary** actions in the page header.

### Example (JS in Page module)

```javascript
frappe.pages["my-dashboard"].on_page_load = function (wrapper) {
  const page = frappe.ui.make_app_page({
    parent: wrapper,
    title: __("My Dashboard"),
    single_column: true,
  });

  page.set_primary_action(__("Run Sync"), () => {
    frappe.call({ method: "my_app.dashboard.run_sync", freeze: true });
  });

  page.set_secondary_action(__("Download CSV"), () => {
    window.open("/api/method/my_app.dashboard.export_csv");
  });

  page.add_inner_button(
    __("Help"),
    () => {
      frappe.msgprint(__("See the docs at /app/knowledge-base"));
    },
    __("More"),
  );
};
```

---

## Common Patterns & Best Practices

- **Visibility rules**

  ```javascript
  if (frm.doc.status !== "Draft") {
    // show button only after submit
  }
  ```

- **Disable conditionally**

  ```javascript
  const b = frm.add_custom_button(__("Approve"), () => {
    /*...*/
  });
  if (!frappe.user.has_role("Approver")) b.prop("disabled", true);
  ```

- **Group actions**

  ```javascript
  frm.add_custom_button(__("Action A"), () => {}, __("Batch"));
  frm.add_custom_button(__("Action B"), () => {}, __("Batch"));
  frm.page.set_inner_btn_group_as_primary(__("Batch"));
  ```

- **Feedback & UX**

  - Use `freeze: true` during long calls.
  - Finish with `frappe.msgprint`, `frappe.show_alert`, and `frm.reload_doc()` if data changed.

- **Error handling**

  ```javascript
  frappe.call({
    method: "...",
    args: {},
    callback(r) {
      if (r.exc) {
        frappe.msgprint(__("Something went wrong."));
      }
    },
  });
  ```

---

## Troubleshooting

- **Button not showing**

  - Ensure your **Client Script** references the correct **DocType**.
  - Check the **`refresh`** event actually runs and the script is **enabled**.
  - Guard against `frm.is_new()` if you only want the button on saved docs.

- **Click does nothing**

  - Open browser console for errors.
  - Confirm server method is **whitelisted** and import path is correct.
  - Verify CSRF/session: use `frappe.call` (it’s session-aware).

- **Permissions errors**

  - Add server-side `frappe.has_permission` checks.
  - Ensure roles/perm levels allow the operation you’re attempting.

---

## Minimal Templates (Copy-Ready)

### Form Toolbar (minimal)

```javascript
frappe.ui.form.on("Your DocType", {
  refresh(frm) {
    if (frm.is_new()) return;
    frm
      .add_custom_button(__("Do Something"), () => {
        frappe.call({
          method: "your_app.your_method",
          args: { name: frm.doc.name },
        });
      })
      .addClass("btn-primary");
  },
});
```

### Button Field (minimal)

```javascript
frappe.ui.form.on("Your DocType", {
  my_button(frm) {
    frappe.msgprint(__("Clicked!"));
  },
});
```

### List View (minimal)

```javascript
frappe.listview_settings["Your DocType"] = {
  onload(listview) {
    listview.page.add_inner_button(__("Bulk Action"), () => {
      const names = listview.get_checked_items().map((d) => d.name);
      if (!names.length) return frappe.msgprint(__("Select rows first."));
      frappe.call({ method: "your_app.bulk_action", args: { names } });
    });
  },
};
```

### Whitelisted Method (minimal)

```python
@frappe.whitelist()
def your_method(name: str):
    doc = frappe.get_doc('Your DocType', name)
    # ... do work ...
    return {'message': 'ok'}
```

---

## Version Notes

- Snippets above are safe for **Frappe v13+** (including v14/v15).
- Exact UI helpers may evolve; prefer `frm.add_custom_button`, `page.add_inner_button`, and `page.set_primary_action` as stable APIs.

---

**That’s it!** Use the pattern that matches your context (form, list, or page), wire it to a whitelisted method for real work, and keep permission checks on the server.
