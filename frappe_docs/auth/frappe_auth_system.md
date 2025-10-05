# Frappe Authentication and Authorization System

## Table of Contents

1. [Authentication System](#authentication-system)
2. [Authorization System](#authorization-system)
3. [API Authentication](#api-authentication)
4. [Security Features](#security-features)
5. [File References](#file-references)

---

## Authentication System

### Overview

Frappe uses a multi-layered authentication system supporting session-based auth, API keys, OAuth 2.0, and custom authentication hooks.

### 1.1 Login Flow

The `LoginManager` class handles all authentication:

```python
class LoginManager:
    def __init__(self):
        if frappe.local.form_dict.get("cmd") == "login":
            self.login()  # New login
        else:
            self.make_session(resume=True)  # Resume existing session
```

**Login Process:**

1. Validate user credentials (`authenticate()`)
2. Check if password reset required
3. Run 2FA if enabled
4. Create session and set cookies
5. Run post-login hooks

**Logout Process:**

```python
def logout(self, arg="", user=None):
    self.run_trigger("on_logout")
    delete_session(frappe.session.sid, user=user)
    self.clear_cookies()
    self.login_as_guest()
```

### 1.2 Session Management

Sessions are stored in two places:

- **Database:** `Sessions` DocType table
- **Redis Cache:** For performance (`frappe.cache.hset("session", sid, data)`)

**Session Data Structure:**

```python
{
    "sid": "generated_hash",
    "user": "user@example.com",
    "data": {
        "last_updated": "2024-01-15 10:30:00",
        "session_expiry": "2024-01-25 10:30:00",
        "full_name": "John Doe",
        "user_type": "System User"
    }
}
```

**Session Creation:**

```python
def start(self):
    sid = frappe.generate_hash()  # Cryptographically secure
    self.data.update({
        "session_expiry": get_expiry_period(),  # Default: 240 hours
        "full_name": self.full_name,
        "user_type": self.user_type
    })
    self.insert_session_record()
```

**Session Expiry:**

- Default: 240 hours (10 days)
- Configurable in System Settings
- Automatically checked on each request
- Expired sessions deleted automatically

### 1.3 Password Security

**Hashing Algorithms:**

- PBKDF2-SHA256 (default)
- Argon2 (modern alternative)

```python
passlibctx = CryptContext(
    schemes=["pbkdf2_sha256", "argon2"]
)
```

**Password Storage:**

- **NEVER** stored in User doctype
- Stored in separate `__Auth` table
- Fields: `doctype`, `name`, `fieldname`, `password` (hashed), `encrypted`

**Password Operations:**

```python
# Hashing (auth.py:117-154)
def update_password(user, pwd):
    hashPwd = passlibctx.hash(pwd)
    frappe.qb.into(Auth)
        .columns(Auth.doctype, Auth.name, Auth.fieldname, Auth.password)
        .insert("User", user, "password", hashPwd)

# Verification (auth.py:78-108)
def check_password(user, pwd):
    result = frappe.qb.from_(Auth).select(Auth.password)
        .where(Auth.name == user).run()

    if not passlibctx.verify(pwd, result[0].password):
        raise frappe.AuthenticationError("Incorrect User or Password")

    # Auto-rehash if algorithm updated
    if passlibctx.needs_update(result[0].password):
        update_password(user, pwd)
```

### 1.4 Multi-Factor Authentication (2FA)

**Supported Methods:**

1. **OTP App** - TOTP using pyotp (Google Authenticator, Authy)
2. **Email** - HOTP sent via email
3. **SMS** - HOTP sent via SMS

**2FA Flow:**

```python
# 1. Check if enabled
def should_run_2fa(user):
    return two_factor_is_enabled(user=user)

# 2. Generate OTP
def authenticate_for_2factor(user):
    otp_secret = get_otpsecret_for_(user)
    token = int(pyotp.TOTP(otp_secret).now())
    tmp_id = frappe.generate_hash(length=8)
    cache_2fa_data(user, token, otp_secret, tmp_id)

# 3. Verify OTP
def confirm_otp_token(login_manager, otp, tmp_id):
    otp_secret = frappe.cache.get(tmp_id + "_otp_secret")

    # Try HOTP (Email/SMS)
    if hotp.verify(otp, hotp_token):
        return True

    # Try TOTP (OTP App)
    if totp.verify(otp):
        return True
```

**OTP Secret Storage:**

- Encrypted in `__DefaultValue` table
- Key: `{user}_otpsecret`
- Parent: `__2fa`

### 1.5 API Authentication Methods

Frappe supports multiple API authentication methods:

#### 1. Session Cookie (Default)

```http
Cookie: sid=session_id; full_name=John
```

#### 2. API Key/Secret (Basic Auth)

```http
Authorization: Basic base64(api_key:api_secret)
```

#### 3. API Key/Secret (Token)

```http
Authorization: token api_key:api_secret
```

#### 4. OAuth 2.0 Bearer Token

```http
Authorization: Bearer access_token
```

**API Key Validation:**

```python
def validate_api_key_secret(api_key, api_secret):
    # Get user with this API key
    docname = frappe.db.get_value(
        doctype="User",
        filters={"api_key": api_key, "enabled": True},
        fieldname="name"
    )

    # Verify secret
    doc_secret = get_decrypted_password("User", docname, "api_secret")
    if doc_secret == api_secret:
        frappe.set_user(frappe.db.get_value("User", docname, "user"))
```

### 1.6 OAuth 2.0 Implementation

**Primary Files:**

- `/frappe/frappe/oauth.py`
- `/frappe/frappe/integrations/oauth2.py`

**Supported Features:**

- Authorization Code flow
- PKCE (Proof Key for Code Exchange)
- Refresh tokens
- Scopes
- OpenID Connect (OIDC) with id_token

**OAuth Flow:**

```
1. Client requests authorization
   GET /api/method/frappe.integrations.oauth2.authorize

2. User approves, redirect with auth code

3. Client exchanges code for token
   POST /api/method/frappe.integrations.oauth2.get_token

4. API requests with Bearer token
   Authorization: Bearer {access_token}
```

### 1.7 Cookie Management

**Cookies Set:**

```python
def init_cookies(self):
    # Session ID (HttpOnly, Secure)
    self.set_cookie("sid", frappe.session.sid,
                    max_age=get_expiry_in_seconds(),
                    httponly=True)

    # User info (accessible to JS)
    self.set_cookie("full_name", self.full_name)
    self.set_cookie("user_id", self.user)
    self.set_cookie("user_image", self.info.user_image)
    self.set_cookie("system_user", "yes" if is_system_user else "no")
```

**Security Features:**

- `HttpOnly` flag on session cookies (prevents XSS access)
- `Secure` flag auto-enabled on HTTPS
- `SameSite=Lax` policy (CSRF protection)
- Configurable max age

### 1.8 CSRF Protection

```python
def validate_csrf_token(self):
    if frappe.request.method in ["POST", "PUT", "DELETE", "PATCH"]:
        csrf_token = frappe.get_request_header("X-Frappe-CSRF-Token")

        if csrf_token != frappe.session.data.csrf_token:
            frappe.throw("Invalid Request", frappe.CSRFTokenError)
```

**CSRF Token:**

- Generated per session
- Must be included in all unsafe HTTP methods (POST, PUT, DELETE, PATCH)
- Header: `X-Frappe-CSRF-Token`
- Bypassed for API key/OAuth authentication

### 1.9 Login Attempt Tracking

**Features:**

- Track failed login attempts
- Lock account after max attempts (default: 3)
- Time-based lock interval (default: 5 minutes)
- Track by username AND IP address

```python
class LoginAttemptTracker:
    def __init__(self, key, max_consecutive_login_attempts=3, lock_interval=300):
        self.key = key
        self.lock_interval = timedelta(seconds=lock_interval)
        self.max_failed_logins = max_consecutive_login_attempts

    def add_failure_attempt(self):
        # Increment counter in cache

    def is_user_allowed(self):
        # Check if user can attempt login
```

---

## Authorization System

### Overview

Frappe uses Role-Based Access Control (RBAC) with additional layers: User Permissions, Document Permissions, and Share Permissions.

### 2.1 Permission Layers (in order)

```
1. Administrator Check → Full access
   ↓
2. Role Permissions → Based on user's roles
   ↓
3. If Owner Permissions → Special rights for document creator
   ↓
4. User Permissions → Document-level restrictions
   ↓
5. Share Permissions → Document shared with user
   ↓
6. Controller Permissions → Custom logic in DocType
```

### 2.2 Role-Based Access Control (RBAC)

**Automatic Roles:**

- `Guest` - All users (including anonymous)
- `All` - All logged-in users
- `Desk User` - System users with desk access
- `Administrator` - Super admin (all permissions)

**Getting User Roles:**

```python
def get_roles(user):
    if user == "Administrator":
        return frappe.get_all("Role", pluck="name")  # All roles

    # Query Has Role table
    roles = frappe.qb.from_(HasRole)
        .where((HasRole.parent == user) & (HasRole.parenttype == "User"))
        .select(HasRole.role)

    # Add automatic roles
    roles += ["Guest", "All"]
    if is_system_user(user):
        roles.append("Desk User")

    return roles
```

### 2.3 Permission Types (Rights)

**Available Rights:** (permissions.py:12-27)

- `select` - Can query in lists
- `read` - Can view document
- `write` - Can modify document
- `create` - Can create new document
- `delete` - Can delete document
- `submit` - Can submit document
- `cancel` - Can cancel submitted document
- `amend` - Can amend cancelled document
- `print` - Can print document
- `email` - Can email document
- `report` - Can generate reports
- `import` - Can import data
- `export` - Can export data
- `share` - Can share document with others

### 2.4 Permission Storage

#### DocPerm (Base Permissions)

Stored in DocType JSON definition:

```json
{
  "permissions": [
    {
      "role": "Sales User",
      "permlevel": 0,
      "read": 1,
      "write": 1,
      "create": 1,
      "delete": 0,
      "if_owner": 0
    }
  ]
}
```

#### Custom DocPerm

Stored in `Custom DocPerm` DocType - overrides base permissions.

#### User Permission

Stored in `User Permission` DocType:

```python
{
    "user": "user@example.com",
    "allow": "Company",              # DocType to restrict
    "for_value": "Wind Power LLC",   # Specific document
    "applicable_for": "Sales Order", # Where to apply (optional)
    "is_default": 1,
    "hide_descendants": 0
}
```

### 2.5 Permission Check Flow

**Primary Function:** `has_permission()` (permissions.py:77-193)

```python
def has_permission(doctype, ptype="read", doc=None, user=None):
    # 1. Administrator always allowed
    if user == "Administrator":
        return True

    # 2. Get user's roles
    roles = frappe.get_roles(user)

    # 3. Check document-level permissions
    if doc:
        perms = get_doc_permissions(doc, user=user, ptype=ptype)
    else:
        perms = get_role_permissions(meta, user=user)

    # 4. Check share permissions if no access
    if not perms.get(ptype):
        perms = check_share_permissions(doc, user)

    return perms.get(ptype, False)
```

### 2.6 Document-Level Permissions

**Function:** `get_doc_permissions()` (permissions.py:196-248)

```python
def get_doc_permissions(doc, user=None, ptype=None):
    # 1. Controller permissions (custom logic)
    if not has_controller_permissions(doc, ptype, user):
        return {ptype: 0}

    # 2. Role permissions
    is_owner = doc.owner == user
    perms = get_role_permissions(meta, user=user, is_owner=is_owner)

    # 3. If Owner permissions
    if perms.get("has_if_owner_enabled") and is_owner:
        perms.update(perms.get("if_owner", {}))

    # 4. User permissions (document restrictions)
    if not has_user_permission(doc, user):
        if is_owner:
            perms = perms.get("if_owner", {})
        else:
            perms = {}

    return perms
```

**If Owner Permissions:**
Special permissions granted only to the document creator (owner field).

Example:

```json
{
  "role": "Sales User",
  "read": 0,
  "write": 0,
  "if_owner": 1 // Only owner can read/write
}
```

### 2.7 Field-Level Permissions

Controlled by **Permission Levels (permlevel)**:

- `permlevel 0` - Default, checked with base permissions
- `permlevel 1+` - Requires specific role permission at that level

**Example:**

```json
{
  "fieldname": "salary",
  "permlevel": 1
}
```

Only roles with permlevel 1 access can read/write this field.

**Permission Check:**

```python
permlevel = meta.get_field(fieldname).permlevel
accessible_permlevels = meta.get_permlevel_access(ptype, user=user)

if permlevel not in accessible_permlevels:
    # Hide field or deny access
```

### 2.8 User Permissions

**Primary File:** `/frappe/frappe/core/doctype/user_permission/user_permission.py`

Restricts which specific documents a user can access:

```python
# User "john@example.com" can only access these companies
User Permission:
  - user: john@example.com
    allow: Company
    for_value: Wind Power LLC

  - user: john@example.com
    allow: Company
    for_value: Solar Power Inc
```

**Implementation:**

```python
def get_user_permissions(user):
    # Returns:
    # {
    #     "Company": [
    #         {"doc": "Wind Power LLC", "is_default": 1},
    #         {"doc": "Solar Power Inc", "is_default": 0}
    #     ]
    # }

    # Cached in Redis: user_permissions:{user}
    return frappe.cache.hget("user_permissions", user)
```

**Applied To:**

- Direct document access
- Link field filters in forms
- List view filters
- Reports

### 2.9 Share Permissions

Users can share specific documents with others:

```python
# Share via DocShare DocType
frappe.share.add(doctype, name, user, write=0, share=0, notify=1)
```

**Shared Document Access:**

```python
def get_shared(doctype, user):
    # Query DocShare table
    return frappe.get_all("DocShare",
        filters={
            "share_doctype": doctype,
            "user": user
        },
        fields=["share_name", "read", "write", "share"]
    )
```

### 2.10 Controller Permissions

Custom permission logic in DocType controllers:

```python
# In a DocType controller class
def has_permission(doc, ptype="read", user=None):
    """Custom permission check"""

    # Example: Only allow if status is Draft
    if ptype == "write" and doc.status != "Draft":
        return False

    # Example: Manager can always access
    if "Manager" in frappe.get_roles(user):
        return True

    return True  # Default allow
```

**Hook Registration:**

```python
# In hooks.py
has_permission = {
    "Sales Order": "myapp.overrides.sales_order.has_permission"
}
```

### 2.11 Permission Caching

**Multi-Level Cache:**

1. **Request Cache** (frappe.local):

   ```python
   frappe.local.role_permissions[cache_key] = perms
   ```

2. **Redis Cache**:

   ```python
   frappe.cache.hset("user_permissions", user, permissions)
   frappe.cache.hset("bootinfo", user, boot_info)
   ```

3. **Database** (fallback):
   Query `DocPerm`, `Custom DocPerm`, `User Permission` tables

**Cache Invalidation:**

- User permissions: On `User Permission` save/delete
- Role permissions: On `DocPerm`/`Custom DocPerm` change
- Session data: On logout/password change

---

## API Authentication

### 3.1 Request Processing Flow

```python
@Request.application
def application(request):
    # 1. Initialize request context
    init_request(request)

    # 2. Validate authentication
    validate_auth()  # ← Authentication happens here

    # 3. Route and handle request
    if frappe.form_dict.cmd:
        frappe.handler.handle()

    # 4. Return response
```

### 3.2 Authentication Validation

```python
def validate_auth():
    auth_header = frappe.get_request_header("Authorization", "").split(" ")

    if len(auth_header) == 2:
        # Try OAuth
        validate_oauth(auth_header)

        # Try API Key/Secret
        validate_auth_via_api_keys(auth_header)

    # Try custom auth hooks
    validate_auth_via_hooks()

    # If still Guest, authentication failed
    if len(auth_header) == 2 and frappe.session.user == "Guest":
        raise frappe.AuthenticationError
```

**Authentication Priority:**

1. OAuth Bearer Token
2. API Key/Secret (Basic or Token)
3. Custom hooks
4. Session cookie (sid)
5. Guest (if allowed)

### 3.3 Whitelist Decorator

All public API methods must be whitelisted:

```python
@frappe.whitelist(allow_guest=False, xss_safe=False, methods=None)
def my_api_method(param1, param2):
    """API method accessible via /api/method/myapp.module.my_api_method"""
    return {"result": "success"}
```

**Parameters:**

- `allow_guest=True` - Allow unauthenticated access
- `xss_safe=True` - Skip XSS sanitization (use with allow_guest)
- `methods=["GET", "POST"]` - Restrict HTTP methods

**Whitelist Check:**

```python
def is_whitelisted(method):
    # Check if method in whitelist
    if method not in whitelisted:
        raise PermissionError("Function not whitelisted")

    # Check guest access
    if frappe.session.user == "Guest" and method not in guest_methods:
        raise PermissionError("Not permitted")

    # XSS sanitization for guest methods
    if is_guest and method not in xss_safe_methods:
        frappe.form_dict = sanitize_html(frappe.form_dict)
```

### 3.4 HTTP Method Validation

```python
def is_valid_http_method(method):
    allowed = frappe.allowed_http_methods_for_whitelisted_func.get(method, [])

    if frappe.request.method not in allowed:
        frappe.throw("Method Not Allowed", frappe.PermissionError)
```

**Default Allowed Methods:**

- GET, POST, PUT, DELETE (if not specified in @whitelist)

### 3.5 API Request Examples

#### Session Cookie Auth

```bash
curl -X POST https://example.com/api/method/frappe.auth.get_logged_user \
  -H "Cookie: sid=session_id" \
  -H "X-Frappe-CSRF-Token: csrf_token"
```

#### API Key Auth (Basic)

```bash
curl -X GET https://example.com/api/method/myapp.api.get_data \
  -H "Authorization: Basic $(echo -n 'api_key:api_secret' | base64)"
```

#### API Key Auth (Token)

```bash
curl -X GET https://example.com/api/method/myapp.api.get_data \
  -H "Authorization: token api_key:api_secret"
```

#### OAuth Bearer Token

```bash
curl -X GET https://example.com/api/method/myapp.api.get_data \
  -H "Authorization: Bearer access_token"
```

---

## Security Features

### 4.1 Password Security

- ✅ PBKDF2-SHA256 or Argon2 hashing
- ✅ Separate `__Auth` table (never in User doctype)
- ✅ Automatic rehashing on algorithm updates
- ✅ Password expiry enforcement
- ✅ Password strength validation
- ✅ Configurable password policies

### 4.2 Session Security

- ✅ Cryptographically secure session IDs
- ✅ HttpOnly cookies for session tokens
- ✅ CSRF token validation for unsafe methods
- ✅ Session expiry enforcement
- ✅ IP-based validation
- ✅ Simultaneous session limits
- ✅ Secure and SameSite cookie attributes

### 4.3 API Security

- ✅ Multiple authentication methods
- ✅ HTTP method restrictions
- ✅ Mandatory whitelist for public methods
- ✅ XSS sanitization for guest methods
- ✅ OAuth 2.0 with PKCE
- ✅ Rate limiting support
- ✅ API key rotation support

### 4.4 Access Control

- ✅ Multi-level permissions (Role → User Permission → Share)
- ✅ Document-level and field-level controls
- ✅ Owner-based permissions
- ✅ Custom permission controllers
- ✅ Permission audit logging
- ✅ Granular permission types (14 rights)

### 4.5 Attack Prevention

- ✅ CSRF protection
- ✅ XSS sanitization
- ✅ SQL injection prevention (Query Builder)
- ✅ Brute force protection (login attempt tracking)
- ✅ Session fixation prevention
- ✅ Secure password storage

---

## Quick Reference

### Check if User Has Permission

```python
# Check DocType permission
if frappe.has_permission("Sales Order", "read"):
    # User can read Sales Orders

# Check specific document permission
doc = frappe.get_doc("Sales Order", "SO-0001")
if frappe.has_permission("Sales Order", "write", doc):
    # User can write this specific document
```

### Get Current User

```python
current_user = frappe.session.user
user_roles = frappe.get_roles()
```

### Create API Method

```python
@frappe.whitelist()
def my_api_method(param1):
    # Requires authentication
    return {"result": param1}

@frappe.whitelist(allow_guest=True)
def public_api():
    # No authentication required
    return {"message": "Hello World"}
```

### Generate API Keys

```python
from frappe.core.doctype.user.user import generate_keys

api_key, api_secret = generate_keys("user@example.com")
```

### Custom Permission Logic

```python
# In DocType controller
def has_permission(doc, ptype="read", user=None):
    if ptype == "write" and doc.status == "Closed":
        return False
    return True
```

---

## Diagram: Complete Authentication Flow

```
HTTP Request
    ↓
┌─────────────────────────────────────┐
│ 1. HTTPRequest Class (auth.py:35)  │
│    - Set request IP                 │
│    - Parse cookies                  │
│    - Initialize session             │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ 2. validate_auth() (auth.py:610)   │
│    Check Authorization header:      │
│    - OAuth Bearer token?            │
│    - API Key/Secret?                │
│    - Custom auth hooks?             │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ 3. LoginManager (auth.py:101)      │
│    If no header auth:               │
│    - Resume session from cookie     │
│    - Or login with credentials      │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ 4. Set User Context                 │
│    frappe.session.user = user       │
│    frappe.local.session_obj = sess  │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ 5. Route Request                    │
│    /api/method/path.to.method       │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ 6. Whitelist Check (handler.py:64) │
│    - Is method whitelisted?         │
│    - Guest access allowed?          │
│    - Sanitize if needed             │
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│ 7. Permission Check                 │
│    has_permission(doctype, ptype)   │
└─────────────────────────────────────┘
    ↓
Execute Method → Return Response
```

## Diagram: Permission Check Hierarchy

```
has_permission(doctype, ptype, doc, user)
    ↓
┌──────────────────────────────┐
│ 1. Administrator?            │ → YES → Allow
└──────────────────────────────┘
    ↓ NO
┌──────────────────────────────┐
│ 2. Get User Roles            │
│    - Direct roles            │
│    - Auto roles (All, Guest) │
└──────────────────────────────┘
    ↓
┌──────────────────────────────┐
│ 3. Get Role Permissions      │
│    - From DocPerm            │
│    - From Custom DocPerm     │
└──────────────────────────────┘
    ↓
┌──────────────────────────────┐
│ 4. If checking specific doc: │
│    a. Controller permission? │
│    b. Is user owner?         │
│    c. Apply if_owner perms   │
│    d. Check user permissions │
│    e. Check link restrictions│
└──────────────────────────────┘
    ↓
┌──────────────────────────────┐
│ 5. No permission yet?        │
│    Check share permissions   │
└──────────────────────────────┘
    ↓
Return True/False or raise PermissionError
```

---

_This documentation is based on Frappe Framework v15. File paths and line numbers may vary in different versions._
