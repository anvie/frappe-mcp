# Frappe Dialog Documentation

## Overview

The Frappe Dialog system provides a powerful and flexible way to create
modal dialogs in Frappe applications. Dialogs are used for user input,
confirmations, displaying information, and various interactive workflows.

## Core Classes

### 1. `frappe.ui.Dialog`

The main class for creating dialogs. It extends `frappe.ui.FieldGroup`,
which in turn extends `frappe.ui.form.Layout`.

**Location**: `/app/frappe/public/js/frappe/ui/dialog.js`

```javascript
frappe.ui.Dialog = class Dialog extends frappe.ui.FieldGroup
```

## Basic Usage

### Creating a Simple Dialog

```javascript
let d = new frappe.ui.Dialog({
    title: 'Enter Details',
    fields: [
        {
            label: 'First Name',
            fieldname: 'first_name',
            fieldtype: 'Data'
        },
        {
            label: 'Last Name',
            fieldname: 'last_name',
            fieldtype: 'Data'
        }
    ],
    primary_action_label: 'Submit',
    primary_action(values) {
        console.log(values);
        d.hide();
    }
});

d.show();
```

## Dialog Options

### Core Configuration Options

| Option                   | Type     | Description                                                            | Default    |
| ------------------------ | -------- | ---------------------------------------------------------------------- | ---------- |
| `title`                  | String   | Dialog title displayed in header                                       | -          |
| `fields`                 | Array    | Array of field definitions                                             | `[]`       |
| `size`                   | String   | Dialog size: `'small'`, `'large'`, `'extra-large'`, or auto-determined | Auto       |
| `static`                 | Boolean  | Makes dialog non-dismissible (no backdrop click/ESC)                   | `false`    |
| `animate`                | Boolean  | Enable fade animation                                                  | `true`     |
| `minimizable`            | Boolean  | Allow dialog to be minimized                                           | `false`    |
| `indicator`              | String   | Color indicator class for header                                       | -          |
| `primary_action`         | Function | Callback for primary button click                                      | -          |
| `primary_action_label`   | String   | Label for primary button                                               | `'Submit'` |
| `secondary_action`       | Function | Callback for secondary button                                          | -          |
| `secondary_action_label` | String   | Label for secondary button                                             | -          |
| `no_submit_on_enter`     | Boolean  | Disable Enter key submission                                           | `false`    |
| `no_focus`               | Boolean  | Disable auto-focus on first input                                      | `false`    |

## Field Types and Configurations

### Commonly Used Field Types

```javascript
fields: [
    // Text Input
    {
        label: 'Name',
        fieldname: 'name',
        fieldtype: 'Data',
        reqd: 1,  // Required field
        default: 'John Doe'
    },

    // Select/Dropdown
    {
        label: 'Status',
        fieldname: 'status',
        fieldtype: 'Select',
        options: ['Draft', 'Submitted', 'Cancelled'],
        default: 'Draft'
    },

    // Link Field (DocType reference)
    {
        label: 'User',
        fieldname: 'user',
        fieldtype: 'Link',
        options: 'User',
        filters: { enabled: 1 }
    },

    // Date Field
    {
        label: 'Start Date',
        fieldname: 'start_date',
        fieldtype: 'Date',
        default: frappe.datetime.get_today()
    },

    // Checkbox
    {
        label: 'Active',
        fieldname: 'is_active',
        fieldtype: 'Check',
        default: 1
    },

    // Text Area
    {
        label: 'Description',
        fieldname: 'description',
        fieldtype: 'Text'
    },

    // Small Text
    {
        label: 'Notes',
        fieldname: 'notes',
        fieldtype: 'Small Text'
    },

    // Password Field
    {
        label: 'Password',
        fieldname: 'password',
        fieldtype: 'Password'
    },

    // Code Editor
    {
        label: 'Script',
        fieldname: 'script',
        fieldtype: 'Code',
        options: 'JavaScript'  // Language for syntax highlighting
    },

    // HTML Field
    {
        fieldtype: 'HTML',
        fieldname: 'preview',
        options: '<p>HTML content here</p>'
    },

    // Section Break
    {
        fieldtype: 'Section Break',
        label: 'Additional Information'
    },

    // Column Break
    {
        fieldtype: 'Column Break'
    }
]
```

### Field Dependencies

```javascript
fields: [
    {
        label: 'Action',
        fieldname: 'action',
        fieldtype: 'Select',
        options: ['Create', 'Edit']
    },
    {
        label: 'Document Name',
        fieldname: 'doc_name',
        fieldtype: 'Data',
        depends_on: doc => doc.action === 'Edit',
        mandatory_depends_on: doc => doc.action === 'Edit'
    }
]
```

## Dialog Methods

### Core Methods

```javascript
// Show the dialog
dialog.show();

// Hide the dialog
dialog.hide();

// Set dialog title
dialog.set_title('New Title');

// Get field values
let values = dialog.get_values();

// Set field value
dialog.set_value('fieldname', 'value');

// Set multiple values
dialog.set_values({
    field1: 'value1',
    field2: 'value2'
});

// Clear all fields
dialog.clear();

// Set primary action
dialog.set_primary_action('Save', function(values) {
    // Handle action
});

// Set secondary action
dialog.set_secondary_action(function() {
    // Handle secondary action
});

// Disable/Enable primary button
dialog.disable_primary_action();
dialog.enable_primary_action();

// Add custom button
dialog.add_custom_action('Custom Action', function() {
    // Handle custom action
}, 'btn-warning');

// Set message
dialog.set_message('Processing...');

// Clear message
dialog.clear_message();

// Get specific field
let field = dialog.get_field('fieldname');

// Set field property
dialog.set_df_property('fieldname', 'read_only', 1);
```

## Event Handlers

### Dialog Events

```javascript
new frappe.ui.Dialog({
    title: 'Dialog with Events',
    fields: [...],

    // Called when dialog is hidden
    onhide: function() {
        console.log('Dialog hidden');
    },

    // Alternative syntax
    on_hide: function() {
        console.log('Dialog hidden');
    },

    // Called when dialog is shown
    on_page_show: function() {
        console.log('Dialog shown');
    },

    // Primary action handler
    primary_action: function(values) {
        console.log('Primary action triggered', values);
    }
});
```

## Advanced Features

### 1. Static/Modal Dialog

```javascript
// Non-dismissible dialog
let d = new frappe.ui.Dialog({
    title: 'Important Action',
    static: true,  // Cannot be closed by clicking backdrop or ESC
    fields: [...],
    primary_action_label: 'Confirm',
    primary_action(values) {
        // Process and then hide
        d.hide();
    }
});
```

### 2. Minimizable Dialog

```javascript
let d = new frappe.ui.Dialog({
    title: 'Background Task',
    minimizable: true,
    fields: [...],
    on_minimize_toggle: function(is_minimized) {
        console.log('Dialog minimized:', is_minimized);
    }
});
```

### 3. Dialog with Size Control

```javascript
// Small dialog
let small_dialog = new frappe.ui.Dialog({
    title: 'Small Dialog',
    size: 'small',
    fields: [...]
});

// Large dialog
let large_dialog = new frappe.ui.Dialog({
    title: 'Large Dialog',
    size: 'large',
    fields: [...]
});

// Extra large dialog
let xl_dialog = new frappe.ui.Dialog({
    title: 'Extra Large Dialog',
    size: 'extra-large',
    fields: [...]
});
```

### 4. Dialog with Validation

```javascript
let d = new frappe.ui.Dialog({
    title: 'User Registration',
    fields: [
        {
            label: 'Email',
            fieldname: 'email',
            fieldtype: 'Data',
            reqd: 1
        },
        {
            label: 'Password',
            fieldname: 'password',
            fieldtype: 'Password',
            reqd: 1
        },
        {
            label: 'Confirm Password',
            fieldname: 'confirm_password',
            fieldtype: 'Password',
            reqd: 1
        }
    ],
    primary_action_label: 'Register',
    primary_action(values) {
        // Custom validation
        if (values.password !== values.confirm_password) {
            frappe.msgprint(__('Passwords do not match'));
            return;
        }

        if (!values.email.includes('@')) {
            frappe.msgprint(__('Invalid email address'));
            return;
        }

        // Process registration
        frappe.call({
            method: 'app.api.register_user',
            args: values,
            callback: function(r) {
                d.hide();
                frappe.msgprint(__('Registration successful'));
            }
        });
    }
});
```

## Specialized Dialogs

### 1. MultiSelectDialog

Used for selecting multiple items from a list with search and filter capabilities.

```javascript
new frappe.ui.form.MultiSelectDialog({
    doctype: 'User',
    target: frm,
    setters: {
        enabled: 1,
        user_type: 'System User'
    },
    add_filters_group: 1,
    primary_action_label: 'Add Users',
    action(selections) {
        console.log('Selected users:', selections);
    }
});
```

### 2. Quick Entry Dialog

For quick creation of documents:

```javascript
frappe.ui.form.make_quick_entry('Customer', function(doc) {
    console.log('Created:', doc);
});
```

## Helper Functions

### frappe.prompt()

A simplified way to create input dialogs:

```javascript
frappe.prompt([
    {
        label: 'Name',
        fieldname: 'name',
        fieldtype: 'Data',
        reqd: 1
    },
    {
        label: 'Email',
        fieldname: 'email',
        fieldtype: 'Data',
        reqd: 1
    }
],
function(values) {
    console.log(values.name, values.email);
},
'Enter Details',
'Submit'
);
```

### frappe.confirm()

For confirmation dialogs:

```javascript
frappe.confirm(
    'Are you sure you want to proceed?',
    function() {
        // User clicked Yes
        console.log('Confirmed');
    },
    function() {
        // User clicked No
        console.log('Cancelled');
    }
);
```

## Best Practices

### 1. Field Validation

Always validate user input before processing:

```javascript
primary_action(values) {
    // Check required fields
    if (!values.fieldname) {
        frappe.throw(__('Field is required'));
    }

    // Validate format
    if (values.email && !frappe.utils.validate_email(values.email)) {
        frappe.msgprint(__('Invalid email format'));
        return;
    }

    // Process...
}
```

### 2. Loading States

Show loading state during async operations:

```javascript
primary_action(values) {
    dialog.set_message('Processing...');
    dialog.disable_primary_action();

    frappe.call({
        method: 'some.method',
        args: values,
        callback: function(r) {
            dialog.clear_message();
            dialog.enable_primary_action();
            dialog.hide();
        },
        error: function() {
            dialog.clear_message();
            dialog.enable_primary_action();
        }
    });
}
```

### 3. Responsive Layouts

Use Section and Column Breaks for better layouts:

```javascript
fields: [
    {
        label: 'Personal Information',
        fieldtype: 'Section Break'
    },
    {
        label: 'First Name',
        fieldname: 'first_name',
        fieldtype: 'Data',
        reqd: 1
    },
    {
        fieldtype: 'Column Break'
    },
    {
        label: 'Last Name',
        fieldname: 'last_name',
        fieldtype: 'Data',
        reqd: 1
    },
    {
        label: 'Contact Information',
        fieldtype: 'Section Break'
    },
    {
        label: 'Email',
        fieldname: 'email',
        fieldtype: 'Data'
    }
]
```

## Common Use Cases

### 1. Data Entry Form

```javascript
function show_data_entry_dialog() {
    let d = new frappe.ui.Dialog({
        title: 'New Customer',
        fields: [
            {
                label: 'Customer Name',
                fieldname: 'customer_name',
                fieldtype: 'Data',
                reqd: 1
            },
            {
                label: 'Email',
                fieldname: 'email',
                fieldtype: 'Data',
                options: 'Email'
            },
            {
                label: 'Phone',
                fieldname: 'phone',
                fieldtype: 'Data',
                options: 'Phone'
            },
            {
                label: 'Customer Group',
                fieldname: 'customer_group',
                fieldtype: 'Link',
                options: 'Customer Group',
                default: 'Individual'
            }
        ],
        primary_action_label: 'Create',
        primary_action(values) {
            frappe.db.insert('Customer', values).then(doc => {
                frappe.show_alert({
                    message: __('Customer {0} created', [doc.name]),
                    indicator: 'green'
                });
                d.hide();
            });
        }
    });
    d.show();
}
```

### 2. File Upload Dialog

```javascript
function show_upload_dialog() {
    let d = new frappe.ui.Dialog({
        title: 'Upload File',
        fields: [
            {
                label: 'File',
                fieldname: 'file',
                fieldtype: 'Attach'
            },
            {
                label: 'Description',
                fieldname: 'description',
                fieldtype: 'Small Text'
            }
        ],
        primary_action_label: 'Upload',
        primary_action(values) {
            if (!values.file) {
                frappe.msgprint(__('Please select a file'));
                return;
            }
            // Process file upload
            d.hide();
        }
    });
    d.show();
}
```

### 3. Search and Select Dialog

```javascript
function show_search_dialog() {
    let d = new frappe.ui.Dialog({
        title: 'Search Items',
        fields: [
            {
                label: 'Search',
                fieldname: 'search_term',
                fieldtype: 'Data',
                onchange: function() {
                    // Trigger search
                    perform_search(this.get_value());
                }
            },
            {
                fieldtype: 'HTML',
                fieldname: 'results_area'
            }
        ]
    });

    function perform_search(term) {
        frappe.call({
            method: 'frappe.desk.search.search_link',
            args: {
                doctype: 'Item',
                txt: term
            },
            callback: function(r) {
                let html = r.results.map(item =>
                    `<div class="result-item" data-value="${item.value}">
                        ${item.label}
                    </div>`
                ).join('');

                d.fields_dict.results_area.$wrapper.html(html);
            }
        });
    }

    d.show();
}
```

## Integration with Forms

### Adding Dialog from Form Button

```javascript
frappe.ui.form.on('Sales Order', {
    refresh(frm) {
        frm.add_custom_button(__('Quick Add Items'), function() {
            let d = new frappe.ui.Dialog({
                title: 'Add Items',
                fields: [
                    {
                        label: 'Item',
                        fieldname: 'item_code',
                        fieldtype: 'Link',
                        options: 'Item',
                        reqd: 1
                    },
                    {
                        label: 'Quantity',
                        fieldname: 'qty',
                        fieldtype: 'Float',
                        default: 1,
                        reqd: 1
                    }
                ],
                primary_action_label: 'Add',
                primary_action(values) {
                    frm.add_child('items', {
                        item_code: values.item_code,
                        qty: values.qty
                    });
                    frm.refresh_field('items');
                    d.hide();
                }
            });
            d.show();
        });
    }
});
```

## Global Dialog Access

The currently active dialog can be accessed via:

```javascript
window.cur_dialog  // Current active dialog
frappe.ui.open_dialogs  // Array of all open dialogs
```

## Styling and Customization

### Custom CSS Classes

```javascript
let d = new frappe.ui.Dialog({
    title: 'Styled Dialog',
    fields: [
        {
            label: 'Custom Field',
            fieldname: 'custom',
            fieldtype: 'Data',
            css_class: 'my-custom-class'
        }
    ]
});

// Add custom styles
d.$wrapper.find('.modal-dialog').addClass('custom-dialog-class');
```

### Indicator Colors

```javascript
let d = new frappe.ui.Dialog({
    title: 'Status Dialog',
    indicator: 'green',  // or 'red', 'orange', 'blue', etc.
    fields: [...]
});
```

## Performance Considerations

1. **Lazy Loading**: Load heavy content only when needed
1. **Debounce Search**: Use debouncing for search fields
1. **Dispose Properly**: Remove event listeners when dialog is hidden
1. **Minimize Field Count**: Use pagination or progressive disclosure for many fields

## Troubleshooting

### Common Issues and Solutions

1. **Dialog not showing**: Ensure `d.show()` is called
1. **Values not captured**: Check field names match exactly
1. **Validation not working**: Return from primary_action to prevent closing
1. **Memory leaks**: Properly clean up event listeners in `onhide`
1. **Focus issues**: Use `no_focus: true` if auto-focus causes problems

## Summary

The Frappe Dialog system provides a comprehensive solution for creating modal
interfaces in Frappe applications. With support for various field types,
validation, event handling, and customization options, it enables developers
to create rich, interactive user experiences while maintaining consistency
with the Frappe framework's design patterns.

