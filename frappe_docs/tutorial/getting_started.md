# Getting Started with Frappe Framework

## Installation

### Prerequisites
- Python 3.10+
- Node.js 18+
- MariaDB 10.3+ or PostgreSQL
- Redis
- wkhtmltopdf (for PDF generation)

### Install Frappe Bench

```bash
# Install frappe-bench
pip3 install frappe-bench

# Initialize bench
bench init frappe-bench --frappe-branch version-15

# Change directory
cd frappe-bench
```

## Creating Your First App

### Step 1: Create New App

```bash
# Create app
bench new-app myapp

# Answer the prompts:
# App Title: My App
# App Description: My first Frappe app
# App Publisher: Your Name
# App Email: your@email.com
# App Icon: icon-name
# App License: MIT
```

### Step 2: Create a Site

```bash
# Create new site
bench new-site mysite.local

# Add your app to the site
bench --site mysite.local install-app myapp

# Set site as default (optional)
bench use mysite.local
```

### Step 3: Start Development Server

```bash
# Start bench
bench start

# In another terminal, watch for changes
bench watch
```

Visit http://mysite.local:8000 to see your site.

## Creating Your First DocType

### Step 1: Using Bench Command

```bash
bench --site mysite.local new-doctype "Task"
```

### Step 2: Or Use the UI

1. Login as Administrator
2. Go to DocType List
3. Click "New"
4. Enter DocType details:
   - Name: Task
   - Module: My App
   - Fields:
     - title (Data, Mandatory)
     - description (Text Editor)
     - status (Select: Open, In Progress, Completed)
     - priority (Select: Low, Medium, High)
     - due_date (Date)

### Step 3: Add Controller Logic

Create `myapp/myapp/doctype/task/task.py`:

```python
import frappe
from frappe.model.document import Document

class Task(Document):
    def validate(self):
        if self.due_date and self.due_date < frappe.utils.today():
            frappe.throw("Due date cannot be in the past")
    
    def on_update(self):
        if self.status == "Completed":
            self.send_completion_email()
    
    def send_completion_email(self):
        # Send email notification
        frappe.sendmail(
            recipients=[frappe.session.user],
            subject=f"Task '{self.title}' Completed",
            message=f"The task '{self.title}' has been marked as completed."
        )
```

## Creating a Simple API

### Step 1: Create API File

Create `myapp/myapp/api.py`:

```python
import frappe

@frappe.whitelist()
def get_open_tasks():
    """Get all open tasks"""
    return frappe.db.get_list("Task",
        filters={"status": "Open"},
        fields=["name", "title", "priority", "due_date"],
        order_by="due_date asc"
    )

@frappe.whitelist()
def create_task(title, description=None, priority="Medium"):
    """Create a new task"""
    task = frappe.get_doc({
        "doctype": "Task",
        "title": title,
        "description": description,
        "priority": priority,
        "status": "Open"
    })
    task.insert()
    frappe.db.commit()
    return task.name
```

### Step 2: Test Your API

```bash
# Using curl
curl -X POST "http://mysite.local:8000/api/method/myapp.api.get_open_tasks" \
  -H "Authorization: token api_key:api_secret"

# Or in Python
import requests

response = requests.post(
    "http://mysite.local:8000/api/method/myapp.api.create_task",
    data={"title": "New Task", "priority": "High"},
    headers={"Authorization": "token api_key:api_secret"}
)
```

## Creating a Web Page

### Step 1: Create Page Structure

```bash
bench --site mysite.local new-page tasks --app myapp
```

### Step 2: Add HTML Template

Create `myapp/www/tasks.html`:

```html
{% extends "templates/web.html" %}

{% block title %}Tasks{% endblock %}

{% block page_content %}
<div class="container">
    <h1>My Tasks</h1>
    
    <div class="row">
        {% for task in tasks %}
        <div class="col-md-4 mb-3">
            <div class="card">
                <div class="card-body">
                    <h5 class="card-title">{{ task.title }}</h5>
                    <p class="card-text">
                        <span class="badge badge-{{ task.priority|lower }}">
                            {{ task.priority }}
                        </span>
                        <br>Due: {{ task.due_date }}
                    </p>
                </div>
            </div>
        </div>
        {% endfor %}
    </div>
</div>
{% endblock %}
```

### Step 3: Add Python Context

Create `myapp/www/tasks.py`:

```python
import frappe

def get_context(context):
    context.tasks = frappe.db.get_list("Task",
        filters={"status": ["!=", "Completed"]},
        fields=["title", "priority", "due_date"],
        order_by="due_date asc"
    )
    return context
```

## Common Bench Commands

```bash
# App Management
bench new-app <app-name>
bench get-app <git-url>
bench install-app <app-name>
bench uninstall-app <app-name>

# Site Management
bench new-site <site-name>
bench drop-site <site-name>
bench backup
bench restore

# Development
bench start
bench watch
bench build
bench migrate
bench clear-cache

# Production
bench setup production
bench setup nginx
bench setup supervisor
bench restart

# Database
bench mariadb
bench mysql
bench console
```

## Project Structure

```
frappe-bench/
├── apps/
│   └── myapp/
│       ├── myapp/
│       │   ├── doctype/
│       │   ├── public/
│       │   ├── templates/
│       │   └── www/
│       ├── hooks.py
│       └── setup.py
├── sites/
│   └── mysite.local/
│       ├── site_config.json
│       └── private/
└── config/
```

## Next Steps

1. **Learn DocTypes**: Create complex DocTypes with relationships
2. **Master the API**: Build REST APIs for external integrations
3. **Client Scripts**: Add JavaScript for dynamic forms
4. **Reports**: Create Query and Script reports
5. **Workflows**: Design approval workflows
6. **Print Formats**: Customize document printing
7. **Web Forms**: Create public-facing forms
8. **Background Jobs**: Implement async processing

## Useful Resources

- Official Documentation: https://frappeframework.com
- GitHub: https://github.com/frappe/frappe
- Community Forum: https://discuss.frappe.io
- Tutorial Videos: https://frappe.school