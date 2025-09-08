# Frappe Framework Documentation

Welcome to the Frappe Framework documentation. Frappe is a full-stack web application framework written in Python and JavaScript.

## Getting Started

Frappe Framework is the backbone of ERPNext and provides a robust foundation for building business applications.

### Key Features

- **DocTypes**: Schema-based data models with automatic CRUD operations
- **REST API**: Automatic REST API generation for all DocTypes
- **Permissions**: Role-based access control system
- **Workflows**: Visual workflow builder for business processes
- **Reports**: Built-in reporting engine with query and script reports
- **Background Jobs**: Async task processing with RQ
- **Email Integration**: Send and receive emails within the framework

## Architecture Overview

Frappe follows an MVC architecture with:
- Models defined as DocTypes (JSON schema)
- Controllers in Python (.py files)
- Views using Desk (single-page application)

## Core Concepts

1. **Sites**: Multi-tenant architecture where each site is a separate database
2. **Apps**: Modular applications that can be installed on sites
3. **Bench**: Command-line tool for managing Frappe deployments
4. **DocTypes**: Data models with built-in ORM
5. **Documents**: Instances of DocTypes

## Development Workflow

1. Create a new app using `bench new-app`
2. Define DocTypes through the UI or JSON files
3. Write business logic in Python controllers
4. Customize forms and views using JavaScript
5. Deploy using bench commands

For detailed information, explore the specific sections of this documentation.