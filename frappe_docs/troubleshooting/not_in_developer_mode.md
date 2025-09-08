# Error not in Developer Mode

When you encounter error messages like this:

"frappe.core.doctype.doctype.doctype.CannotCreateStandardDoctypeError: Not in Developer Mode! Set in site_config.json or make 'Custom' DocType."

Then you need to enable developer mode for current site, by running the
following command:

```bash
bench --site yoursite.local set-config developer_mode 1
```
