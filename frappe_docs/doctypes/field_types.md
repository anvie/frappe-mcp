# Frappe Field Types

## Overview

Frappe provides various field types to handle different data requirements. Each field type has specific properties and behaviors.

## Text Fields

### Data
- Single line text input
- Max length: 140 characters by default
- Options: Set length with `length` property

### Text
- Multi-line text input
- No length limit
- Rendered as textarea

### Small Text
- Multi-line text for smaller content
- Stored as TEXT in database

### Long Text
- For large text content
- Stored as LONGTEXT in database

### Text Editor
- Rich text editor with formatting
- Stores HTML content
- Includes toolbar for formatting

### Code
- Syntax highlighted code editor
- Options: Set language (python, javascript, json, html, css)

### Password
- Masked input field
- Encrypted storage
- Not visible in list views

## Numeric Fields

### Int
- Integer values only
- No decimal places
- Range: -2147483648 to 2147483647

### Float
- Decimal numbers
- Precision: Set with `precision` property
- Default precision: 9

### Currency
- Monetary values
- Automatic formatting based on currency
- Options: Set currency code

### Percent
- Percentage values
- Stored as decimal (0-100)
- Displayed with % symbol

## Date and Time Fields

### Date
- Date picker
- Format: YYYY-MM-DD
- Default options: Today, Yesterday

### Datetime
- Date and time picker
- Format: YYYY-MM-DD HH:MM:SS
- Timezone aware

### Time
- Time picker only
- Format: HH:MM:SS
- 24-hour format

### Duration
- Time duration input
- Format: D days H hours M minutes S seconds

## Selection Fields

### Select
- Dropdown with predefined options
- Options: Newline separated values
```
Option 1
Option 2
Option 3
```

### Link
- Reference to another DocType
- Creates foreign key relationship
- Options: Target DocType name

### Dynamic Link
- Reference to any DocType
- Requires link_fieldname
- Flexible relationships

### Table
- Child table for one-to-many relations
- Options: Child DocType name
- Inline editing support

### Table MultiSelect
- Multiple selection from child table
- Many-to-many relationships

## File Fields

### Attach
- File upload field
- Stores file in File doctype
- Supports all file types

### Attach Image
- Image upload only
- Preview in form
- Image optimization

### Image
- Image URL or selection
- Can be external URL
- Preview support

### Signature
- Digital signature pad
- Stores as base64 image

## Boolean Fields

### Check
- Checkbox input
- Values: 0 or 1
- Default: 0

## Special Fields

### Button
- Actionable button in form
- Triggers client-side events
- No data storage

### Column Break
- Layout control
- Creates new column in form
- No data storage

### Section Break
- Layout control
- Creates new section
- Can be collapsible

### Tab Break
- Creates tabbed interface
- Organizes fields in tabs

### HTML
- Custom HTML content
- For display only
- No data storage

### Heading
- Section heading
- Formatting only
- No data storage

### Read Only
- Display-only field
- Computed values
- Options: Set default value or fetch from

### Geolocation
- Latitude and longitude
- Map integration
- Location picker

### JSON
- JSON data storage
- Object/array support
- JSON editor interface

### Markdown Editor
- Markdown text editor
- Live preview
- Stored as markdown text

### Rating
- Star rating input
- Scale: 1-5 by default
- Visual rating display

### Color
- Color picker
- Hex color values
- Visual color selector

## Field Properties

### Common Properties
- **Label**: Display name
- **Fieldname**: Database column name
- **Fieldtype**: Type of field
- **Mandatory** (reqd): Required field
- **Read Only**: Non-editable
- **Hidden**: Not visible in form
- **Default**: Default value
- **Description**: Help text
- **Depends On**: Show/hide based on condition
- **Fetch From**: Auto-populate from linked document
- **Fetch If Empty**: Only fetch if field is empty

### Validation Properties
- **Unique**: Enforce unique values
- **Set Only Once**: Cannot be changed after creation
- **Allow in Quick Entry**: Show in quick entry form
- **Ignore User Permissions**: Bypass permission checks

## Examples

### Creating a Link Field with Filters
```json
{
  "fieldname": "customer",
  "fieldtype": "Link",
  "label": "Customer",
  "options": "Customer",
  "reqd": 1,
  "filters": {
    "disabled": 0,
    "customer_group": "Retail"
  }
}
```

### Conditional Field Display
```json
{
  "fieldname": "discount_amount",
  "fieldtype": "Currency",
  "label": "Discount Amount",
  "depends_on": "eval:doc.apply_discount==1"
}
```

### Fetch From Example
```json
{
  "fieldname": "customer_email",
  "fieldtype": "Data",
  "label": "Email",
  "fetch_from": "customer.email_id",
  "read_only": 1
}
```