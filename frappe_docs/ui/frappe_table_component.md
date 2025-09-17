# Creating Custom Tables in Frappe Pages

## Overview

This guide demonstrates how to create custom tables with Frappe field controls in custom pages. This approach provides flexibility while leveraging Frappe's powerful field components.

## Table of Contents

1. [Basic Structure](#basic-structure)
2. [Adding Frappe Field Controls](#adding-frappe-field-controls)
3. [Event Handling](#event-handling)
4. [Data Management](#data-management)
5. [Validation](#validation)
6. [Complete Example](#complete-example)

## Basic Structure

### HTML Table Template

```javascript
setup_table() {
    const table_html = $(`
        <div class="form-section">
            <h4>Table Title</h4>
            <div class="custom-table-container">
                <table class="table table-bordered" id="custom-table">
                    <thead>
                        <tr>
                            <th style="width: 5%">#</th>
                            <th style="width: 30%">Column 1</th>
                            <th style="width: 25%">Column 2</th>
                            <th style="width: 20%">Column 3</th>
                            <th style="width: 10%">Column 4</th>
                            <th style="width: 10%">Actions</th>
                        </tr>
                    </thead>
                    <tbody id="table-tbody">
                    </tbody>
                </table>
                <div class="table-actions">
                    <button class="btn btn-sm btn-primary" id="add-row-btn">
                        <i class="fa fa-plus"></i> Add Row
                    </button>
                </div>
            </div>
        </div>
    `).appendTo(this.container);

    // Initialize data array
    this.table_rows = [];

    // Setup add button
    $('#add-row-btn').on('click', () => {
        this.add_table_row();
    });

    // Add initial row
    this.add_table_row();
}
```

## Adding Frappe Field Controls

### Creating a Row with Field Controls

```javascript
add_table_row() {
    // Generate unique row ID
    const row_id = 'row_' + Math.random().toString(36).substring(2, 9);
    const row_index = this.table_rows.length;

    // Create row HTML
    const row_html = $(`
        <tr data-row-id="${row_id}" data-index="${row_index}">
            <td class="text-center">${row_index + 1}</td>
            <td><div id="field1-${row_id}"></div></td>
            <td><div id="field2-${row_id}"></div></td>
            <td><div id="field3-${row_id}"></div></td>
            <td><div id="field4-${row_id}"></div></td>
            <td class="text-center">
                <button class="btn btn-sm btn-danger remove-row-btn" data-row-id="${row_id}">
                    <i class="fa fa-trash"></i>
                </button>
            </td>
        </tr>
    `).appendTo('#table-tbody');

    // Create field controls
    const row_fields = {};

    // Link Field Example
    row_fields.field1 = frappe.ui.form.make_control({
        df: {
            fieldtype: 'Link',
            options: 'DocType Name',  // Replace with actual DocType
            placeholder: 'Select...',
            get_query: () => {
                return {
                    filters: {
                        // Add any filters here
                    }
                };
            },
            change: () => {
                this.handle_field_change(row_id);
            }
        },
        parent: row_html.find(`#field1-${row_id}`)[0],
        render_input: true
    });

    // Data Field Example
    row_fields.field2 = frappe.ui.form.make_control({
        df: {
            fieldtype: 'Data',
            placeholder: 'Enter text',
            change: () => {
                this.handle_field_change(row_id);
            }
        },
        parent: row_html.find(`#field2-${row_id}`)[0],
        render_input: true
    });

    // Float Field Example
    row_fields.field3 = frappe.ui.form.make_control({
        df: {
            fieldtype: 'Float',
            placeholder: '0.00',
            default: 0,
            change: () => {
                this.handle_field_change(row_id);
            }
        },
        parent: row_html.find(`#field3-${row_id}`)[0],
        render_input: true
    });

    // Currency Field Example (Read-only)
    row_fields.field4 = frappe.ui.form.make_control({
        df: {
            fieldtype: 'Currency',
            read_only: 1,
            placeholder: '0.00'
        },
        parent: row_html.find(`#field4-${row_id}`)[0],
        render_input: true
    });

    // Store row data
    this.table_rows.push({
        id: row_id,
        fields: row_fields,
        data: {}
    });

    // Setup remove button
    row_html.find('.remove-row-btn').on('click', () => {
        this.remove_table_row(row_id);
    });
}
```

## Event Handling

### Handling Field Changes

```javascript
handle_field_change(row_id) {
    const row = this.table_rows.find(r => r.id === row_id);
    if (!row) return;

    // Get values from fields
    const value1 = row.fields.field1.get_value();
    const value2 = row.fields.field2.get_value();
    const value3 = parseFloat(row.fields.field3.get_value()) || 0;

    // Store in data object
    row.data.field1 = value1;
    row.data.field2 = value2;
    row.data.field3 = value3;

    // Perform calculations or other logic
    this.calculate_row_values(row_id);
    this.update_totals();
}
```

### Auto-population from Link Field

```javascript
handle_link_field_change(row_id) {
    const row = this.table_rows.find(r => r.id === row_id);
    if (!row) return;

    const selected_value = row.fields.field1.get_value();
    if (!selected_value) {
        // Clear dependent fields
        row.fields.field2.set_value('');
        row.fields.field3.set_value(0);
        return;
    }

    // Fetch related data
    frappe.call({
        method: 'frappe.client.get',
        args: {
            doctype: 'DocType Name',
            name: selected_value
        },
        callback: (r) => {
            if (r.message) {
                const doc = r.message;

                // Update dependent fields
                row.fields.field2.set_value(doc.some_field || '');
                row.fields.field3.set_value(doc.numeric_field || 0);

                // Update data
                row.data.field2 = doc.some_field || '';
                row.data.field3 = doc.numeric_field || 0;

                // Recalculate if needed
                this.calculate_row_values(row_id);
            }
        }
    });
}
```

## Data Management

### Removing Rows

```javascript
remove_table_row(row_id) {
    // Check minimum rows requirement
    if (this.table_rows.length <= 1) {
        frappe.msgprint(__('At least one row is required'));
        return;
    }

    // Remove from DOM
    $(`tr[data-row-id="${row_id}"]`).remove();

    // Remove from data array
    this.table_rows = this.table_rows.filter(r => r.id !== row_id);

    // Update row numbers
    this.update_row_numbers();

    // Recalculate totals
    this.update_totals();
}

update_row_numbers() {
    $('#table-tbody tr').each((index, row) => {
        $(row).find('td:first').text(index + 1);
        $(row).attr('data-index', index);
    });
}
```

### Calculations

```javascript
calculate_row_values(row_id) {
    const row = this.table_rows.find(r => r.id === row_id);
    if (!row) return;

    // Example: Calculate amount = quantity * rate
    const qty = parseFloat(row.fields.quantity?.get_value()) || 0;
    const rate = parseFloat(row.fields.rate?.get_value()) || 0;
    const amount = qty * rate;

    // Update calculated field
    if (row.fields.amount) {
        row.fields.amount.set_value(amount);
    }

    // Store calculated value
    row.data.amount = amount;
}

update_totals() {
    let total = 0;

    this.table_rows.forEach(row => {
        total += parseFloat(row.data.amount) || 0;
    });

    // Display total
    $('#total-amount').text(frappe.format(total, {fieldtype: 'Currency'}));
}
```

## Validation

### Form Validation

```javascript
validate_table_data() {
    const errors = [];
    let has_valid_rows = false;

    this.table_rows.forEach((row, index) => {
        const field1 = row.fields.field1.get_value();
        const field3 = parseFloat(row.fields.field3.get_value()) || 0;

        // Check if row has any data
        if (field1 || field3 > 0) {
            // Validate required fields
            if (!field1) {
                errors.push(`Row ${index + 1}: Field 1 is required`);
            }
            if (field3 <= 0) {
                errors.push(`Row ${index + 1}: Field 3 must be greater than 0`);
            } else {
                has_valid_rows = true;
            }
        }
    });

    if (!has_valid_rows) {
        errors.push('At least one valid row is required');
    }

    if (errors.length > 0) {
        frappe.msgprint({
            title: __('Validation Error'),
            message: errors.join('<br>'),
            indicator: 'red'
        });
        return false;
    }

    return true;
}
```

### Collecting Data for Submission

```javascript
collect_table_data() {
    const data = [];

    this.table_rows.forEach(row => {
        const field1 = row.fields.field1.get_value();
        const field3 = parseFloat(row.fields.field3.get_value()) || 0;

        // Only include rows with valid data
        if (field1 && field3 > 0) {
            data.push({
                field1: field1,
                field2: row.fields.field2.get_value() || '',
                field3: field3,
                field4: row.data.field4 || 0
            });
        }
    });

    return data;
}
```

## Complete Example

### CSS Styles

```css
.custom-table-container {
  margin-bottom: 15px;
}

#custom-table {
  margin-bottom: 10px;
}

#custom-table th {
  background-color: #f8f9fa;
  font-weight: 600;
  border-color: #dee2e6;
}

#custom-table td {
  vertical-align: middle;
  padding: 4px;
}

#custom-table td .control-input {
  margin-bottom: 0;
}

.table-actions {
  margin-bottom: 10px;
}

.total-section {
  border-top: 2px solid #f0f0f0;
  padding-top: 15px;
  text-align: right;
}
```

## Available Field Types

Frappe provides various field types you can use in your custom table:

- **Data**: Plain text input
- **Link**: Dropdown with autocomplete to select DocType records
- **Select**: Dropdown with predefined options
- **Float**: Numeric input for decimal numbers
- **Int**: Numeric input for integers
- **Currency**: Formatted currency input
- **Date**: Date picker
- **Datetime**: Date and time picker
- **Time**: Time picker
- **Check**: Checkbox
- **Text**: Multi-line text input
- **Small Text**: Smaller multi-line text input
- **Color**: Color picker
- **Rating**: Star rating input

### Field Definition Options

```javascript
{
    fieldtype: 'FieldType',      // Required: Type of field
    fieldname: 'field_name',      // Optional: Internal field name
    label: 'Field Label',         // Optional: Display label
    placeholder: 'Placeholder',   // Optional: Placeholder text
    default: 'default_value',     // Optional: Default value
    reqd: 1,                      // Optional: Required field (0 or 1)
    read_only: 1,                 // Optional: Read-only field (0 or 1)
    options: 'DocType/Options',   // For Link: DocType name, For Select: Options
    change: () => {},             // Optional: Change event handler
    get_query: () => {},          // Optional: For Link fields - filter query
    depends_on: 'condition',      // Optional: Show/hide based on condition
}
```

## Best Practices

1. **Always initialize data structures** before creating the table
2. **Use unique IDs** for each row to avoid conflicts
3. **Implement proper validation** before processing data
4. **Handle edge cases** like minimum row requirements
5. **Clean up event listeners** when removing rows
6. **Use Frappe's formatting utilities** for displaying values
7. **Implement loading states** for async operations
8. **Provide clear error messages** for validation failures
9. **Consider keyboard navigation** for better UX
10. **Test with various data scenarios** including empty states

## Tips

- Use `frappe.format()` to display formatted values
- Use `frappe.msgprint()` for user notifications
- Use `frappe.call()` for server interactions
- Leverage Frappe's built-in validation for field types
- Consider implementing undo/redo functionality for better UX
- Add keyboard shortcuts for common actions (e.g., Ctrl+Enter to add row)
- Implement auto-save functionality for long forms
- Consider pagination for tables with many rows

This approach provides a flexible solution that works well in Frappe custom pages without the complexity of the built-in grid system.
