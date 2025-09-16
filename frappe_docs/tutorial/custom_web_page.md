# Building Custom Web Pages in Frappe

This guide explains how to create public-facing web pages in a Frappe app using the `www/` directory. It covers folder structure, routing, templating with Jinja, dynamic context via Python, assets, SEO, caching, and common pitfalls—with copy‑pasteable examples.

---

## When to Use `www/`

Use `www/` when you want **highly-customized web page** that are public by default. Typical cases:

- Static pages (landing pages, docs, marketing pages)
- Lightly dynamic pages rendered with Jinja variables from `get_context`
- Pages that don’t need database‑driven routes (for that, consider **Website Generator** doctypes)

Prefer [**Frappe Custom Page**](custom_web_page.md) using Web Page doctype or **Website Generator** when non‑developers will edit content or when you need database‑backed, user‑editable pages.

---

## Directory & Routing Basics

Place pages inside your app at `your_app/www/`.

**Routing rules:**

- `www/hello.html` → `/hello`
- `www/hello.md` → `/hello`
- `www/hello/index.html` → `/hello/`
- Nested folders mirror URL paths: `www/docs/getting-started.html` → `/docs/getting-started`
- If both `.md` and `.html` exist for the same route, Frappe resolves one; avoid duplicates.

```
your_app/
  www/
    index.html          # → /
    hello.html          # → /hello
    hello.py            # optional: dynamic context for /hello
    docs/
      getting-started.md  # → /docs/getting-started
      index.html          # → /docs/
```

> **Note:** `www` routes are public. Protect data access in any code you call from them.

---

## Page Types

### 1) Plain HTML (`.html`)

Write standard HTML. You can still use Jinja tags if you extend a base template (see below).

**Example:** `www/hello.html`

```html
{% extends "templates/web.html" %} {% block page_content %}
<section class="hero">
  <h1>Hello from www</h1>
  <p>This is a custom page served at <code>/hello</code>.</p>
</section>
{% endblock %}
```

### 2) Markdown with Frontmatter (`.md`)

Markdown pages support YAML frontmatter for metadata and template selection.

**Example:** `www/docs/getting-started.md`

```markdown
---
title: Getting Started
base_template: templates/web.html
---

# Getting Started

This page lives in `www/docs/getting-started.md` and renders at `/docs/getting-started`.
```

> **Common frontmatter keys:** `title`, `base_template`. (Advanced: you can define your own keys and read them in `get_context`.)

---

## Adding Dynamic Data with `get_context`

Create a Python file **with the same basename** as your page to inject variables into the template.

**Example files:**

```
www/
  hello.html
  hello.py
```

**`www/hello.py`**

```python
# Runs before rendering /hello

def get_context(context):
    context.greeting = "Assalamualaikum & Hello, World!"
    context.features = [
        {"title": "Fast", "desc": "Served directly from your app."},
        {"title": "Simple", "desc": "No database record needed."},
        {"title": "Customizable", "desc": "Use Jinja and your own CSS/JS."},
    ]
    # Control caching for this page (optional)
    context.no_cache = 1  # set 0 or remove for cacheable pages
```

**`www/hello.html`**

```html
{% extends "templates/web.html" %} {% block page_content %}
<h1>{{ greeting }}</h1>
<ul>
  {% for f in features %}
  <li><strong>{{ f.title }}</strong> — {{ f.desc }}</li>
  {% endfor %}
</ul>
{% endblock %}
```

> **Tip:** Any attribute you add to `context` in `get_context` becomes available in your Jinja template.

---

## Page Layouts & Jinja Blocks

Frappe’s default website base is `templates/web.html`. Common blocks:

- `page_content`: your main content
- `title`: page `<title>` (or set via frontmatter `title`)
- `head`: inject extra `<meta>` / `<link>`

**Example:**

```html
{% extends "templates/web.html" %} {% block title %}Custom Title | My Site{%
endblock %} {% block head %}
<meta name="description" content="My custom landing page" />
{% endblock %} {% block page_content %}
<h1>Welcome</h1>
{% endblock %}
```

---

## Including CSS & JS

### Per‑page includes

Put sibling files next to your page and reference them.

```
www/
  landing.html
  landing.css
  landing.js
```

**`landing.html`**

```html
{% extends "templates/web.html" %} {% block head %}
<link rel="stylesheet" href="/assets/{{ frappe.get_app_name() }}/landing.css" />
{% endblock %} {% block page_content %}
<div class="hero">Hello</div>
<script src="/assets/{{ frappe.get_app_name() }}/landing.js"></script>
{% endblock %}
```

> For bundling/minification, prefer placing assets in `your_app/public/` and referencing via `/assets/your_app/...`. You can also declare global includes in `hooks.py` using `web_include_css` and `web_include_js`.

### Global includes via `hooks.py`

```python
# in your_app/hooks.py
web_include_css = ["/assets/your_app/css/site.css"]
web_include_js  = ["/assets/your_app/js/site.js"]
```

Place the actual files under `your_app/public/css` and `your_app/public/js`.

---

## SEO & Metadata

- **Title**: set via frontmatter (`title:`) or a `{% block title %}`.
- **Meta description / Open Graph**: inject tags inside `{% block head %}`.
- **Canonical URL**: also add in `{% block head %}` if you need it.
- **Sitemap**: Pages in `www/` are discoverable; you can opt out by setting `context.no_sitemap = 1` in `get_context`.

**Example:**

```html
{% block head %}
<meta name="description" content="Docs for Product X" />
<meta property="og:title" content="Product X Docs" />
<meta property="og:type" content="website" />
{% endblock %}
```

---

## Caching & Performance

- **Page cache**: By default, pages may be cached. To disable for a specific page: `context.no_cache = 1` in `get_context`.
- **Static assets** (under `/assets`) are fingerprinted and cached aggressively.
- **Avoid heavy DB calls** in `get_context` or add caching logic (e.g., memoize, or cache results in `frappe.cache()` and invalidate when needed).

---

## Access Control & Security

- URLs under `www/` are public. Do not expose sensitive data.
- For server actions (form submissions, mutations), use a **whitelisted API method** (`@frappe.whitelist`) in a Python module and call it from your page via `frappe.call`/`fetch`.
- Validate all input on the server. If allowing guests, add `allow_guest=True` and perform your own checks (CSRF, rate‑limits if needed).

**Example API:**

```python
# your_app/api.py
import frappe

@frappe.whitelist(allow_guest=True)
def subscribe(email: str):
    if not frappe.utils.validate_email_address(email, throw=False):
        frappe.throw("Invalid email")
    # store or enqueue
    return {"ok": True}
```

**Client call:**

```html
<script>
  async function subscribe() {
    const email = $("#email").val();
    frappe.call({
      method: "your_app.api.subscribe",
      args: { email },
      callback: (res) => {
        if (res.message && res.message.ok) {
          alert("Subscribed!");
        } else {
          alert("Error subscribing.");
        }
      },
    });
  }
</script>
```

---

## Using Data from Doctypes

Within `get_context`, you can query the database—but keep it fast and safe.

```python
import frappe

def get_context(context):
    posts = frappe.get_all("Blog Post", fields=["name", "route", "title"], limit_page_length=5)
    context.latest_posts = posts
```

```html
<ul>
  {% for p in latest_posts %}
  <li><a href="/{{ p.route }}">{{ p.title }}</a></li>
  {% endfor %}
</ul>
```

---

## Redirects

Create a tiny page that redirects, or use site config/Desk redirects if available.

```html
{% extends "templates/web.html" %} {% block head %}
<meta http-equiv="refresh" content="0; url=/new-path" />
<link rel="canonical" href="/new-path" />
{% endblock %} {% block page_content %}
<p>Redirecting…</p>
{% endblock %}
```

---

## Common Pitfalls

1. **Duplicate routes** between `www/` and Website records → resolve by removing one or changing the path.
2. **Heavy queries in `get_context`** → cache results or paginate.
3. **Forgetting `base_template`** in `.md` pages → no site chrome (header/footer) shows.
4. **Posting directly to a `www` route** → use whitelisted `/api/method/...` endpoints instead.
5. **Missing assets path** → put files under `your_app/public/...` and reference via `/assets/your_app/...`.

---

## Minimal “Hello World” (Copy‑Paste)

1. `your_app/www/hello.py`

```python
def get_context(context):
    context.greeting = "Hello from Frappe www!"
```

2. `your_app/www/hello.html`

```html
{% extends "templates/web.html" %} {% block page_content %}
<h1>{{ greeting }}</h1>
<p>This page lives at <code>/hello</code>.</p>
{% endblock %}
```

Visit: `/hello`

---

## See Also

- **Web Page** doctype (for DB‑backed pages editable in Desk)
- **Website Generator** doctypes (for listing/detail routes)
- **`templates/`** directory for shared includes and base templates
- **`hooks.py`** for global web includes and website settings
