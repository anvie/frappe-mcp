# Frappe Custom Page Documentation: Building Forms with Frappe UI

## Table of Contents

1. [Overview](#overview)
2. [Page Structure](#page-structure)
3. [Creating a Custom Page](#creating-a-custom-page)
4. [Building Forms with Frappe UI](#building-forms-with-frappe-ui)
5. [Backend Integration](#backend-integration)
6. [Field Types and Controls](#field-types-and-controls)
7. [Validation and Dependencies](#validation-and-dependencies)
8. [Styling and UX Patterns](#styling-and-ux-patterns)
9. [Best Practices](#best-practices)
10. [Complete Example](#complete-example)

## Overview

Frappe custom pages are standalone interfaces built using the Frappe framework's UI components. They provide complete control over layout and functionality while leveraging Frappe's powerful form controls, validation, and backend integration capabilities.

### Key Components

- **Page Definition** (JSON): Metadata about the page
- **Python Backend**: Server-side logic and API endpoints
- **JavaScript Frontend**: Client-side UI and interactions
- **Frappe UI Controls**: Built-in form components

## Page Structure

A Frappe custom page consists of four main files:

```
your_module/page/
└── your_page_name/
    ├── __init__.py                 # Python module init
    ├── your_page_name.json         # Page metadata
    ├── your_page_name.py           # Backend logic
    └── your_page_name.js           # Frontend UI
```

### File Descriptions

#### 1. JSON Configuration (`your_page_name.json`)

```json
{
  "doctype": "Page",
  "module": "Your Module",
  "name": "your-page-name", // URL slug
  "page_name": "your-page-name",
  "title": "Your Page Title",
  "standard": "Yes",
  "roles": [
    // Access control
    { "role": "System Manager" },
    { "role": "Your Custom Role" }
  ]
}
```

#### 2. Python Backend (`your_page_name.py`)

```python
import frappe
from frappe import _

def get_context(context):
    """Page context for server-side rendering (optional)"""
    context.no_cache = 1
    context.title = _("Your Page Title")

    # Permission check
    if not frappe.has_permission("DocType", "create"):
        frappe.throw(_("Not permitted"), frappe.PermissionError)

    return context

@frappe.whitelist()
def your_api_method(param1, param2):
    """API endpoint callable from frontend"""
    # Process data
    # Return response
    return {"status": "success", "data": result}
```

#### 3. JavaScript Frontend (`your_page_name.js`)

```javascript
frappe.pages["your-page-name"].on_page_load = function (wrapper) {
  var page = frappe.ui.make_app_page({
    parent: wrapper,
    title: "Your Page Title",
    single_column: true,
  });

  // Initialize your custom class
  wrapper.your_page = new YourPageClass(wrapper);
};
```

## Creating a Custom Page

### Step 1: Create Page Files

Use Frappe's bench command:

```bash
bench new-page your_page_name
```

### Step 2: Define Page Class Structure

```javascript
frappe.YourPageClass = class YourPageClass {
  constructor(wrapper) {
    this.wrapper = wrapper;
    this.page = wrapper.page;
    this.body = $(this.wrapper).find(".main-section");
    this.fields = {}; // Store form field references
    this.make();
  }

  make() {
    this.setup_page();
    this.setup_actions();
  }

  setup_page() {
    // Build your UI here
  }

  setup_actions() {
    // Set page actions/buttons
  }
};
```

## Building Forms with Frappe UI

### Using frappe.ui.form.make_control()

The core pattern for creating form fields in custom pages:

```javascript
// Basic field creation
this.fields.field_name = frappe.ui.form.make_control({
  df: {
    // df = DocField definition
    fieldname: "field_name",
    label: __("Field Label"),
    fieldtype: "Data", // Field type
    reqd: 1, // Required field
    description: "Help text",
  },
  parent: container_element, // DOM element to append to
  render_input: true, // Render immediately
});

// Get field value
const value = this.fields.field_name.get_value();

// Set field value
this.fields.field_name.set_value("new value");

// Clear field
this.fields.field_name.set_value("");
```

### Form Layout Pattern

Create sections with rows and columns:

```javascript
create_form_section() {
    // Create section container
    const section = $(`
        <div class="form-section">
            <h5>Section Title</h5>
            <div class="row"></div>
        </div>
    `).appendTo(this.body);

    const row = section.find('.row');

    // Two-column layout
    const col1 = $('<div class="col-md-6"></div>').appendTo(row);
    const col2 = $('<div class="col-md-6"></div>').appendTo(row);

    // Add fields to columns
    this.fields.field1 = frappe.ui.form.make_control({
        df: { /* field definition */ },
        parent: col1[0],
        render_input: true
    });
}
```

## Backend Integration

### Making API Calls

```javascript
// Call backend method
async submit_form() {
    try {
        const response = await frappe.call({
            method: "module.page.your_page.your_page.your_method",
            args: {
                param1: this.fields.field1.get_value(),
                param2: JSON.stringify(complex_data)
            }
        });

        const result = response.message;

        if (result.status === "success") {
            // Handle success
            frappe.msgprint(__("Success!"));
        } else {
            // Handle error
            frappe.msgprint({
                title: __("Error"),
                indicator: "red",
                message: result.message
            });
        }
    } catch (error) {
        frappe.msgprint(__("An error occurred: ") + error.message);
    }
}
```

### Backend Processing

```python
@frappe.whitelist()
def process_form_data(param1, param2):
    """Process form submission"""
    try:
        # Parse JSON if needed
        if isinstance(param2, str):
            import json
            param2 = json.loads(param2)

        # Begin transaction
        frappe.db.begin()

        try:
            # Create/update documents
            doc = frappe.new_doc("DocType")
            doc.field1 = param1
            doc.insert(ignore_permissions=True)

            # Commit transaction
            frappe.db.commit()

            return {
                "status": "success",
                "message": _("Created successfully"),
                "data": {"name": doc.name}
            }

        except Exception as e:
            frappe.db.rollback()
            raise

    except Exception as e:
        frappe.log_error(f"Error: {str(e)}")
        return {
            "status": "error",
            "message": str(e)
        }
```

## Field Types and Controls

### Common Field Types

```javascript
// Text Input
frappe.ui.form.make_control({
  df: {
    fieldname: "text_field",
    label: "Text Field",
    fieldtype: "Data",
  },
});

// Select/Dropdown
frappe.ui.form.make_control({
  df: {
    fieldname: "select_field",
    label: "Select Field",
    fieldtype: "Select",
    options: "\nOption 1\nOption 2\nOption 3", // Note the leading \n
  },
});

// Link Field (DocType reference)
frappe.ui.form.make_control({
  df: {
    fieldname: "link_field",
    label: "Link Field",
    fieldtype: "Link",
    options: "Customer", // DocType name
  },
});

// Date Field
frappe.ui.form.make_control({
  df: {
    fieldname: "date_field",
    label: "Date Field",
    fieldtype: "Date",
  },
});

// Currency Field
frappe.ui.form.make_control({
  df: {
    fieldname: "amount",
    label: "Amount",
    fieldtype: "Currency",
    default: 0,
  },
});

// Checkbox
frappe.ui.form.make_control({
  df: {
    fieldname: "checkbox_field",
    label: "Checkbox",
    fieldtype: "Check",
  },
});

// Textarea
frappe.ui.form.make_control({
  df: {
    fieldname: "description",
    label: "Description",
    fieldtype: "Small Text",
  },
});
```

## Validation and Dependencies

### Field Validation

```javascript
setup_validations() {
    // Email validation
    this.fields.email.$input.on('blur', () => {
        const email = this.fields.email.get_value();
        if (email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
            frappe.msgprint({
                title: __("Validation Error"),
                message: __("Please enter a valid email"),
                indicator: "red"
            });
            this.fields.email.$input.focus();
        }
    });

    // Custom validation on change
    this.fields.age.$input.on('change', () => {
        const age = this.fields.age.get_value();
        if (age && age < 18) {
            frappe.msgprint(__("Age must be 18 or above"));
        }
    });
}
```

### Cascading/Dependent Fields

```javascript
setup_field_queries() {
    // Set query for dependent field
    this.fields.city.get_query = () => {
        const province = this.fields.province.get_value();
        if (province) {
            return {
                filters: {
                    province: province
                }
            };
        }
    };

    // Clear dependent fields on parent change
    this.fields.province.$input.on('change', () => {
        this.fields.city.set_value("");
        this.fields.district.set_value("");
    });
}
```

### Form-Level Validation

```javascript
validate_form() {
    const errors = [];

    // Check required fields
    const required_fields = [
        {field: 'name', label: 'Name'},
        {field: 'email', label: 'Email'}
    ];

    required_fields.forEach(({field, label}) => {
        const value = this.fields[field].get_value();
        if (!value || value.trim() === '') {
            errors.push(`${label} is required`);
        }
    });

    if (errors.length > 0) {
        frappe.msgprint({
            title: __("Validation Error"),
            message: errors.join("<br>"),
            indicator: "red"
        });
        return false;
    }

    return true;
}
```

## Styling and UX Patterns

### Modern Card-Based Layout

```javascript
// Add custom CSS
$(`<style>
    .form-section {
        background: white;
        padding: 25px;
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.08);
        margin-bottom: 20px;
        transition: transform 0.2s, box-shadow 0.2s;
    }
    
    .form-section:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0,0,0,0.12);
    }
    
    .section-header {
        margin-bottom: 25px;
        padding-bottom: 15px;
        border-bottom: 2px solid #f0f0f0;
    }
    
    .btn-primary {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        border: none;
        border-radius: 6px;
        padding: 10px 24px;
        font-weight: 500;
        transition: all 0.3s;
    }
    
    .btn-primary:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
    }
</style>`).appendTo(page.main);
```

### Section with Icon and Description

```javascript
create_styled_section() {
    const section = $(`
        <div class="form-section">
            <div class="section-header">
                <h5>
                    <i class="fa fa-user" style="color: #667eea; margin-right: 10px;"></i>
                    ${__("Customer Information")}
                </h5>
                <p style="margin: 5px 0 0 30px; color: #7c858e; font-size: 13px;">
                    ${__("Personal details and contact information")}
                </p>
            </div>
            <div class="row"></div>
        </div>
    `).appendTo(this.body);

    return section.find('.row');
}
```

### Success Dialog Pattern

```javascript
show_success_dialog(data) {
    const dialog = new frappe.ui.Dialog({
        title: __('Success!'),
        size: 'large',
        fields: [{
            fieldtype: 'HTML',
            fieldname: 'content',
            options: `
                <div style="text-align: center; padding: 20px;">
                    <i class="fa fa-check-circle" style="font-size: 48px; color: #48bb78;"></i>
                    <h4>${__('Operation Successful')}</h4>
                    <p>${__('Details: ') + data.message}</p>
                </div>
            `
        }],
        primary_action_label: __('OK'),
        primary_action: () => dialog.hide()
    });

    dialog.show();
}
```

## Best Practices

### 1. Structure and Organization

- Keep form sections logically grouped
- Use descriptive field names
- Store field references in `this.fields` object
- Separate validation logic from UI creation

### 2. User Experience

- Provide clear labels and help text
- Show loading states during API calls
- Use appropriate field types (Date picker for dates, etc.)
- Implement real-time validation feedback
- Group related fields together

### 3. Error Handling

```javascript
try {
    // Validate before submission
    if (!this.validate_form()) return;

    // Show loading
    frappe.freeze(__('Processing...'));

    // Make API call
    const response = await frappe.call({...});

    // Handle response
    if (response.message.status === 'success') {
        // Success handling
    } else {
        // Error handling
    }
} catch (error) {
    frappe.msgprint({
        title: __('Error'),
        message: error.message,
        indicator: 'red'
    });
} finally {
    frappe.unfreeze();
}
```

### 4. Performance

- Use field queries to filter Link fields
- Implement debouncing for real-time validation
- Load data asynchronously when possible
- Clear dependent fields to prevent invalid states

### 5. Security

- Always use `@frappe.whitelist()` for exposed methods
- Validate permissions in backend
- Sanitize user inputs
- Use transactions for data consistency
- Never expose sensitive data in client-side code

## Complete Example

Here's a minimal working example of a custom registration form:

### 1. Create Page Structure

```bash
bench new-page user_registration
```

### 2. JSON Configuration

```json
{
  "doctype": "Page",
  "module": "Your Module",
  "name": "user-registration",
  "page_name": "user-registration",
  "title": "User Registration",
  "standard": "Yes",
  "roles": [{ "role": "System Manager" }]
}
```

### 3. JavaScript Implementation

```javascript
frappe.pages["user-registration"].on_page_load = function (wrapper) {
  var page = frappe.ui.make_app_page({
    parent: wrapper,
    title: "User Registration",
    single_column: true,
  });

  wrapper.registration = new UserRegistration(wrapper);
};

class UserRegistration {
  constructor(wrapper) {
    this.wrapper = wrapper;
    this.page = wrapper.page;
    this.body = $(wrapper).find(".page-content");
    this.fields = {};
    this.make();
  }

  make() {
    this.setup_form();
    this.setup_buttons();
  }

  setup_form() {
    const container = $('<div class="registration-form"></div>').appendTo(
      this.body,
    );

    // Name field
    this.fields.full_name = frappe.ui.form.make_control({
      df: {
        fieldname: "full_name",
        label: __("Full Name"),
        fieldtype: "Data",
        reqd: 1,
      },
      parent: container[0],
      render_input: true,
    });

    // Email field
    this.fields.email = frappe.ui.form.make_control({
      df: {
        fieldname: "email",
        label: __("Email"),
        fieldtype: "Data",
        reqd: 1,
      },
      parent: container[0],
      render_input: true,
    });

    // Department field (Link to DocType)
    this.fields.department = frappe.ui.form.make_control({
      df: {
        fieldname: "department",
        label: __("Department"),
        fieldtype: "Link",
        options: "Department",
      },
      parent: container[0],
      render_input: true,
    });
  }

  setup_buttons() {
    this.page.set_primary_action(__("Submit"), () => {
      this.submit_form();
    });

    this.page.set_secondary_action(__("Clear"), () => {
      this.clear_form();
    });
  }

  async submit_form() {
    // Collect data
    const data = {
      full_name: this.fields.full_name.get_value(),
      email: this.fields.email.get_value(),
      department: this.fields.department.get_value(),
    };

    // Validate
    if (!data.full_name || !data.email) {
      frappe.msgprint(__("Please fill all required fields"));
      return;
    }

    // Submit
    try {
      const response = await frappe.call({
        method:
          "your_module.page.user_registration.user_registration.create_user",
        args: { data: data },
      });

      if (response.message.status === "success") {
        frappe.msgprint(__("User created successfully!"));
        this.clear_form();
      }
    } catch (error) {
      frappe.msgprint(__("Error: ") + error.message);
    }
  }

  clear_form() {
    Object.keys(this.fields).forEach((fieldname) => {
      this.fields[fieldname].set_value("");
    });
  }
}
```

### 4. Python Backend

```python
import frappe
from frappe import _

@frappe.whitelist()
def create_user(data):
    """Create a new user record"""
    try:
        import json
        if isinstance(data, str):
            data = json.loads(data)

        # Create user logic here
        user = frappe.new_doc("User")
        user.email = data.get("email")
        user.first_name = data.get("full_name")
        user.insert(ignore_permissions=True)

        frappe.db.commit()

        return {
            "status": "success",
            "message": _("User created successfully"),
            "user": user.name
        }

    except Exception as e:
        frappe.log_error(f"Error creating user: {str(e)}")
        return {
            "status": "error",
            "message": str(e)
        }
```

## Summary

Building custom pages with forms in Frappe follows a consistent pattern:

1. **Structure**: Create page files with JSON config, Python backend, and JavaScript frontend
2. **Form Building**: Use `frappe.ui.form.make_control()` for creating form fields
3. **Layout**: Organize fields in sections with rows and columns
4. **Validation**: Implement both client-side and server-side validation
5. **Backend Integration**: Use `frappe.call()` to communicate with Python methods
6. **Styling**: Apply modern CSS for better UX
7. **Error Handling**: Provide clear feedback for success and failure states

The framework provides powerful tools for building complex forms while maintaining consistency with the rest of the Frappe/ERPNext ecosystem. The key is understanding the patterns and leveraging the built-in components effectively.

