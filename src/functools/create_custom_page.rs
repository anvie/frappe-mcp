// Copyright (C) 2025 Nuwaira
// All Rights Reserved.
//
// NOTICE: All information contained herein is, and remains
// the property of Nuwaira.
// The intellectual and technical concepts contained
// herein are proprietary to Nuwaira
// and are protected by trade secret or copyright law.
// Dissemination of this information or reproduction of this material
// is strictly forbidden unless prior written permission is obtained
// from Nuwaira.
#![allow(dead_code)]
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::{
    analyze::AnalyzedData,
    stringutil::{to_kebabc, to_snakec},
};
use rmcp::{model::*, ErrorData as McpError};

type McpResult = Result<CallToolResult, McpError>;

pub fn create_custom_page(
    config: &Config,
    _anal: &AnalyzedData,
    page_name: &str,
    module: &str,
    title: Option<String>,
    roles: Option<Vec<String>>,
) -> McpResult {
    let page_name_snake = to_snakec(page_name);
    let page_name_kebab = to_kebabc(page_name);
    let module_snake = to_snakec(module);

    let base_dir = format!(
        "{}/{}/{}/page/{}",
        config.app_absolute_path,
        to_snakec(&config.app_name),
        module_snake,
        page_name_snake
    );
    let base_dir = Path::new(&base_dir);

    // Extract title or use page name
    let page_title = title.unwrap_or_else(|| page_name.to_string());
    let page_roles = roles.unwrap_or_else(|| vec!["System Manager".to_string()]);

    // Check if any file already exists
    let json_file = base_dir.join(format!("{}.json", page_name_snake));
    let py_file = base_dir.join(format!("{}.py", page_name_snake));
    let js_file = base_dir.join(format!("{}.js", page_name_snake));
    let init_file = base_dir.join("__init__.py");

    if json_file.exists() || py_file.exists() || js_file.exists() {
        mcp_return!(format!(
            "Custom page '{}' already exists at: {}",
            page_name,
            base_dir.display()
        ));
    }

    // Create parent directories if they don't exist
    if !base_dir.exists() {
        if let Err(e) = fs::create_dir_all(base_dir) {
            mcp_return!(format!(
                "Failed to create directory {}: {}",
                base_dir.display(),
                e
            ));
        }
    }

    let mut result = Vec::new();

    // Create __init__.py
    if let Err(e) = fs::write(&init_file, "") {
        mcp_return!(format!("Failed to write __init__.py file: {}", e));
    }
    result.push(format!("✓ Created __init__.py: {}", init_file.display()));

    // Create JSON configuration file
    let json_content = create_json_boilerplate(&page_name_kebab, module, &page_title, &page_roles);
    if let Err(e) = fs::write(&json_file, json_content) {
        mcp_return!(format!("Failed to write JSON file: {}", e));
    }
    result.push(format!("✓ Created JSON: {}", json_file.display()));

    // Create Python backend file
    let py_content = create_python_boilerplate(&page_title);
    if let Err(e) = fs::write(&py_file, py_content) {
        mcp_return!(format!("Failed to write Python file: {}", e));
    }
    result.push(format!("✓ Created Python: {}", py_file.display()));

    // Create JavaScript frontend file
    let js_content = create_js_boilerplate(&page_name_kebab, &page_title, config);
    if let Err(e) = fs::write(&js_file, js_content) {
        mcp_return!(format!("Failed to write JavaScript file: {}", e));
    }
    result.push(format!("✓ Created JavaScript: {}", js_file.display()));

    let summary = format!(
        "Custom page '{}' created successfully:\n\n{}\n\nNext steps:\n1. Create the Page doctype record in the database:\n   \
            - Go to Page List in the Desk\n   \
            - Create a new Page with:\n     \
                * Name: {}\n     \
                * Module: {}\n     \
                * Standard: Yes\n   \
            - OR use: bench execute \"frappe.get_doc({{'doctype': 'Page', 'name': '{}', 'title': '{}', 'page_name': '{}', 'module': '{}', 'standard': 'Yes'}}).insert()\"\n\n2. Clear cache and reload:\n   - bench clear-cache\n   - Refresh your browser\n\n3. Access your page at: /app/{}\n\n4. Customize the form fields in the JavaScript file\n5. Add backend API methods in the Python file",
        page_title,
        result.join("\n"),
        page_name_kebab,
        module,
        page_name_kebab,
        page_title,
        page_name_kebab,
        module,
        page_name_kebab
    );

    mcp_return!(summary)
}

fn create_json_boilerplate(page_name: &str, module: &str, title: &str, roles: &[String]) -> String {
    let roles_json: Vec<String> = roles
        .iter()
        .map(|role| format!(r#"  {{ "role": "{}" }}"#, role))
        .collect();

    format!(
        r#"{{
 "doctype": "Page",
 "module": "{}",
 "name": "{}",
 "page_name": "{}",
 "title": "{}",
 "standard": "Yes",
 "roles": [
{}
 ]
}}"#,
        module,
        page_name,
        page_name,
        title,
        roles_json.join(",\n")
    )
}

fn create_python_boilerplate(title: &str) -> String {
    format!(
        r#"import frappe
from frappe import _

def get_context(context):
    """Page context for server-side rendering (optional)"""
    context.no_cache = 1
    context.title = _("{}")
    
    # Add permission checks if needed
    # if not frappe.has_permission("DocType", "create"):
    #     frappe.throw(_("Not permitted"), frappe.PermissionError)
    
    return context

@frappe.whitelist()
def submit_form(data):
    """API endpoint for form submission"""
    try:
        import json
        if isinstance(data, str):
            data = json.loads(data)
        
        # Validate data
        if not data.get("name"):
            frappe.throw(_("Name is required"))
        
        # Begin transaction
        frappe.db.begin()
        
        try:
            # Example: Create a new document
            # doc = frappe.new_doc("YourDocType")
            # doc.field1 = data.get("field1")
            # doc.field2 = data.get("field2")
            # doc.insert(ignore_permissions=True)
            
            # Commit transaction
            frappe.db.commit()
            
            return {{
                "status": "success",
                "message": _("Form submitted successfully"),
                "data": {{
                    # Return any relevant data
                }}
            }}
            
        except Exception as e:
            frappe.db.rollback()
            raise
            
    except Exception as e:
        frappe.log_error(f"Error in submit_form: {{str(e)}}", "{}")
        return {{
            "status": "error",
            "message": str(e)
        }}

@frappe.whitelist()
def get_data():
    """API endpoint to fetch data"""
    try:
        # Example: Fetch data from database
        # data = frappe.db.get_list("YourDocType",
        #     fields=["name", "field1", "field2"],
        #     filters={{}},
        #     order_by="creation desc",
        #     limit=10
        # )
        
        return {{
            "status": "success",
            "data": []
        }}
        
    except Exception as e:
        frappe.log_error(f"Error in get_data: {{str(e)}}", "{}")
        return {{
            "status": "error",
            "message": str(e)
        }}
"#,
        title, title, title
    )
}

fn create_js_boilerplate(page_name: &str, title: &str, config: &Config) -> String {
    format!(
        r#"frappe.pages["{}"].on_page_load = function(wrapper) {{
    var page = frappe.ui.make_app_page({{
        parent: wrapper,
        title: "{}",
        single_column: true
    }});

    // Initialize the page
    wrapper.{} = new {}(wrapper);
}};

class {} {{
    constructor(wrapper) {{
        this.wrapper = wrapper;
        this.page = wrapper.page;
        this.body = $(this.wrapper).find('.main-section');
        this.fields = {{}};
        this.make();
    }}

    make() {{
        this.setup_page();
        this.setup_form();
        this.setup_actions();
        this.setup_validations();
    }}

    setup_page() {{
        // Add any custom CSS
        this.add_custom_css();
        
        // Create main container
        this.container = $(`
            <div class="custom-page-container">
                <div class="page-description">
                    <p>Welcome to the {} page. This is a custom Frappe page with form controls.</p>
                </div>
            </div>
        `).appendTo(this.body);
    }}

    setup_form() {{
        // Create form sections
        const form_section = this.create_form_section(
            "Basic Information",
            "Enter the basic details below"
        );
        
        // Example: Name field
        this.fields.full_name = frappe.ui.form.make_control({{
            df: {{
                fieldname: "full_name",
                label: __("Full Name"),
                fieldtype: "Data",
                reqd: 1,
                description: "Enter your full name"
            }},
            parent: form_section[0],
            render_input: true
        }});

        // Example: Email field
        this.fields.email = frappe.ui.form.make_control({{
            df: {{
                fieldname: "email",
                label: __("Email"),
                fieldtype: "Data",
                reqd: 1,
                description: "Enter your email address"
            }},
            parent: form_section[0],
            render_input: true
        }});

        // Example: Select field
        this.fields.department = frappe.ui.form.make_control({{
            df: {{
                fieldname: "department",
                label: __("Department"),
                fieldtype: "Select",
                options: "\\nSales\\nMarketing\\nEngineering\\nSupport",
                description: "Select your department"
            }},
            parent: form_section[0],
            render_input: true
        }});

        // Example: Link field (to DocType)
        this.fields.customer = frappe.ui.form.make_control({{
            df: {{
                fieldname: "customer",
                label: __("Customer"),
                fieldtype: "Link",
                options: "Customer",
                description: "Select a customer"
            }},
            parent: form_section[0],
            render_input: true
        }});

        // Example: Date field
        this.fields.date = frappe.ui.form.make_control({{
            df: {{
                fieldname: "date",
                label: __("Date"),
                fieldtype: "Date",
                default: frappe.datetime.get_today(),
                description: "Select a date"
            }},
            parent: form_section[0],
            render_input: true
        }});

        // Example: Currency field
        this.fields.amount = frappe.ui.form.make_control({{
            df: {{
                fieldname: "amount",
                label: __("Amount"),
                fieldtype: "Currency",
                default: 0,
                description: "Enter amount"
            }},
            parent: form_section[0],
            render_input: true
        }});

        // Example: Text area
        this.fields.description = frappe.ui.form.make_control({{
            df: {{
                fieldname: "description",
                label: __("Description"),
                fieldtype: "Small Text",
                description: "Enter additional details"
            }},
            parent: form_section[0],
            render_input: true
        }});
    }}

    create_form_section(title, description) {{
        const section = $(`
            <div class="form-section">
                <div class="section-header">
                    <h5>${{title}}</h5>
                    <p class="text-muted">${{description}}</p>
                </div>
                <div class="row">
                    <div class="col-md-12"></div>
                </div>
            </div>
        `).appendTo(this.container);

        return section.find('.col-md-12');
    }}

    setup_actions() {{
        // Add primary action button
        this.page.set_primary_action(__("Submit"), () => {{
            this.submit_form();
        }});

        // Add secondary action button
        this.page.set_secondary_action(__("Clear"), () => {{
            this.clear_form();
        }});

        // Add custom button in page menu
        this.page.add_menu_item(__("Refresh Data"), () => {{
            this.refresh_data();
        }});
    }}

    setup_validations() {{
        // Email validation
        this.fields.email.$input.on('blur', () => {{
            const email = this.fields.email.get_value();
            if (email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {{
                frappe.msgprint({{
                    title: __("Validation Error"),
                    message: __("Please enter a valid email address"),
                    indicator: "red"
                }});
                this.fields.email.$input.focus();
            }}
        }});

        // Add more validations as needed
    }}

    async submit_form() {{
        try {{
            // Validate form
            if (!this.validate_form()) {{
                return;
            }}

            // Collect form data
            const data = this.get_form_data();

            // Show loading state
            frappe.freeze(__("Processing..."));

            // Make API call
            const response = await frappe.call({{
                method: "{}.{}.page.{}.{}.submit_form",
                args: {{
                    data: data
                }}
            }});

            // Handle response
            if (response.message.status === "success") {{
                frappe.msgprint({{
                    title: __("Success"),
                    message: response.message.message,
                    indicator: "green"
                }});
                this.clear_form();
            }} else {{
                frappe.msgprint({{
                    title: __("Error"),
                    message: response.message.message,
                    indicator: "red"
                }});
            }}

        }} catch (error) {{
            frappe.msgprint({{
                title: __("Error"),
                message: error.message || __("An error occurred"),
                indicator: "red"
            }});
        }} finally {{
            frappe.unfreeze();
        }}
    }}

    validate_form() {{
        const errors = [];
        
        // Check required fields
        const required_fields = [
            {{field: 'full_name', label: 'Full Name'}},
            {{field: 'email', label: 'Email'}}
        ];

        required_fields.forEach(({{field, label}}) => {{
            const value = this.fields[field].get_value();
            if (!value || value.trim() === '') {{
                errors.push(`${{label}} is required`);
            }}
        }});

        if (errors.length > 0) {{
            frappe.msgprint({{
                title: __("Validation Error"),
                message: errors.join("<br>"),
                indicator: "red"
            }});
            return false;
        }}

        return true;
    }}

    get_form_data() {{
        const data = {{}};
        
        // Collect all field values
        Object.keys(this.fields).forEach(fieldname => {{
            data[fieldname] = this.fields[fieldname].get_value();
        }});

        return data;
    }}

    clear_form() {{
        // Clear all fields
        Object.keys(this.fields).forEach(fieldname => {{
            this.fields[fieldname].set_value("");
        }});
    }}

    async refresh_data() {{
        try {{
            frappe.freeze(__("Loading..."));
            
            const response = await frappe.call({{
                method: "{}.{}.page.{}.{}.get_data",
                args: {{}}
            }});

            if (response.message.status === "success") {{
                // Handle the data
                console.log("Data received:", response.message.data);
                frappe.msgprint(__("Data refreshed successfully"));
            }}

        }} catch (error) {{
            frappe.msgprint({{
                title: __("Error"),
                message: error.message,
                indicator: "red"
            }});
        }} finally {{
            frappe.unfreeze();
        }}
    }}

    add_custom_css() {{
        if (!document.getElementById('{}-custom-css')) {{
            $(`<style id="{}-custom-css">
                .custom-page-container {{
                    padding: 20px;
                }}
                
                .page-description {{
                    background: #f5f7fa;
                    padding: 15px;
                    border-radius: 8px;
                    margin-bottom: 20px;
                }}
                
                .form-section {{
                    background: white;
                    padding: 25px;
                    border-radius: 8px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.08);
                    margin-bottom: 20px;
                }}
                
                .section-header {{
                    margin-bottom: 20px;
                    padding-bottom: 15px;
                    border-bottom: 1px solid #e0e0e0;
                }}
                
                .section-header h5 {{
                    margin: 0;
                    color: #333;
                }}
                
                .section-header p {{
                    margin: 5px 0 0 0;
                    font-size: 13px;
                }}
            </style>`).appendTo('head');
        }}
    }}
}}
"#,
        page_name,
        title,
        to_snakec(page_name),
        title.replace(' ', ""),
        title.replace(' ', ""),
        title,
        config.app_name.replace(' ', "_"),
        to_snakec(&config.app_name),
        page_name,
        page_name,
        config.app_name.replace(' ', "_"),
        to_snakec(&config.app_name),
        page_name,
        page_name,
        page_name,
        page_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::AnalyzedData;
    use crate::config::Config;

    fn mock_config() -> Config {
        Config {
            frappe_bench_dir: "/tmp".to_string(),
            app_name: "test_app".to_string(),
            app_absolute_path: "/tmp/test".to_string(),
            app_relative_path: "test_app".to_string(),
            site: "frontend".to_string(),
        }
    }

    #[test]
    fn test_create_json_boilerplate() {
        let json = create_json_boilerplate(
            "user-settings",
            "Core",
            "User Settings",
            &vec!["System Manager".to_string()],
        );
        assert!(json.contains(r#""doctype": "Page""#));
        assert!(json.contains(r#""module": "Core""#));
        assert!(json.contains(r#""name": "user-settings""#));
        assert!(json.contains(r#""title": "User Settings""#));
        assert!(json.contains(r#""role": "System Manager""#));
    }

    #[test]
    fn test_create_python_boilerplate() {
        let py = create_python_boilerplate("User Settings");
        assert!(py.contains("import frappe"));
        assert!(py.contains("def get_context(context):"));
        assert!(py.contains("@frappe.whitelist()"));
        assert!(py.contains("def submit_form(data):"));
        assert!(py.contains("User Settings"));
    }

    #[test]
    fn test_create_js_boilerplate() {
        let config = mock_config();
        let js = create_js_boilerplate("user-settings", "User Settings", &config);
        assert!(js.contains(r#"frappe.pages["user-settings"]"#));
        assert!(js.contains("class UserSettings"));
        assert!(js.contains("frappe.ui.form.make_control"));
        assert!(js.contains("setup_form()"));
        assert!(js.contains("submit_form()"));
    }

    #[test]
    fn test_create_custom_page() {
        use std::fs;
        use std::path::Path;

        // Create a temporary test directory
        let test_dir = "/tmp/frappe_mcp_test_custom_page";
        let app_path = format!("{}/test_app", test_dir);

        // Clean up any existing test directory
        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir).unwrap();
        }

        let config = Config {
            frappe_bench_dir: test_dir.to_string(),
            app_name: "Test App".to_string(),
            app_absolute_path: app_path.clone(),
            app_relative_path: "test_app".to_string(),
            site: "frontend".to_string(),
        };

        // Create a minimal AnalyzedData instance
        let anal = AnalyzedData {
            doctypes: vec![],
            modules: vec![],
            symbol_refs: None,
        };

        // Test 1: Create custom page
        let result = create_custom_page(
            &config,
            &anal,
            "User Settings",
            "Core",
            Some("User Settings Page".to_string()),
            Some(vec!["System Manager".to_string(), "Employee".to_string()]),
        );
        assert!(result.is_ok());

        // Verify files were created
        let page_dir = Path::new(&app_path).join("test_app/core/page/user_settings");
        assert!(page_dir.exists());
        assert!(page_dir.join("__init__.py").exists());
        assert!(page_dir.join("user_settings.json").exists());
        assert!(page_dir.join("user_settings.py").exists());
        assert!(page_dir.join("user_settings.js").exists());

        // Verify JSON content
        let json_content = fs::read_to_string(page_dir.join("user_settings.json")).unwrap();
        assert!(json_content.contains(r#""name": "user-settings""#));
        assert!(json_content.contains(r#""module": "Core""#));
        assert!(json_content.contains(r#""role": "System Manager""#));
        assert!(json_content.contains(r#""role": "Employee""#));

        // Test 2: Try to create duplicate page
        let result = create_custom_page(&config, &anal, "User Settings", "Core", None, None);
        assert!(result.is_ok());
        if let Ok(tool_result) = result {
            if let Some(first_content) = tool_result.content.first() {
                if let RawContent::Text(text_content) = &first_content.raw {
                    assert!(text_content.text.contains("already exists"));
                }
            }
        }

        // Clean up
        fs::remove_dir_all(test_dir).unwrap();
    }
}
