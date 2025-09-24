# Frappe: Bulk Processing with Queues

This guide shows how to process large datasets in the background using Frappe queues (Redis + RQ).

---

## 0) TL;DR (Copy This First)

1. **Start workers**: `bench worker --queue default,short,long`
2. **Write a task** (idempotent, chunked):

   ```python
   # app_name/app_name/tasks/bulk.py
   import frappe

   def process_records(ids: list[int] | list[str]):
       frappe.db.commit()  # Start with clean connection

       for docname in ids:
           try:
               # Use unique savepoint per document
               savepoint = f"bulk_processing_{docname}"
               frappe.db.savepoint(savepoint)

               doc = frappe.get_doc("ToDo", docname)  # example doctype
               # ... your logic here ...
               doc.save()

           except Exception as e:
               frappe.db.rollback(save_point=savepoint)
               frappe.log_error(f"Bulk queue failed for {docname}: {str(e)}", "bulk.process_records")

       frappe.db.commit()  # Commit successful processing
   ```

3. **Enqueue in chunks**:

   ```python
   # app_name/app_name/api/bulk.py
   import math, frappe

   @frappe.whitelist()
   def enqueue_bulk(ids: list[str] | str, chunk_size: int = 200, queue: str = "long"):
       if isinstance(ids, str):
           # support CSV string from UI
           ids = [x.strip() for x in ids.split(",") if x.strip()]

       total = len(ids)
       for i in range(0, total, chunk_size):
           chunk = ids[i : i + chunk_size]
           frappe.enqueue(
               "app_name.app_name.tasks.bulk.process_records",
               queue=queue,
               job_name=f"bulk:process_records:{i//chunk_size+1}",
               job_id=f"bulk_{frappe.generate_hash(length=16)}_{i//chunk_size+1}",
               timeout=60*30,  # 30m
               at_front=False,
               ids=chunk,
           )
       return {"enqueued": math.ceil(total / chunk_size), "total": total}
   ```

4. **UI call (Client Script example)**:

   ```javascript
   // In a List view button or Custom Button
   frappe
     .call({
       method: "app_name.app_name.api.bulk.enqueue_bulk",
       args: {
         ids: selected_docs.map((d) => d.name),
         chunk_size: 200,
         queue: "long",
       },
     })
     .then((r) => frappe.msgprint(`Enqueued ${r.message.enqueued} jobs`));
   ```

That's the core. Below are the details you'll care about in production.

---

## 1) When to Use Queues for Bulk

Use queues when:

- Processing **hundreds/thousands** of docs
- Work is **slow or I/O heavy** (emails, integrations, PDFs, images)
- You want **non-blocking UX** and **retry isolation** per chunk

Avoid queues when:

- You only update < 50 docs and it's fast
- You need **strong, single-transaction** semantics across all records

---

## 2) Queue Basics in Frappe

- **Queues**: `short`, `default`, `long`. Pick by expected execution time
- **Workers**: processes that consume jobs. Start multiple per queue
- **Enqueue API**: `frappe.enqueue(fn, queue=..., timeout=..., job_name=..., **kwargs)`
- **Where**: Put tasks under `app_name/app_name/tasks/` for clarity

### Start Workers

```bash
# Development - single terminal
bench worker --queue default,short,long

# Development - separate terminals (better for debugging)
bench worker --queue short
bench worker --queue default
bench worker --queue long

# Production (supervisor)
bench setup supervisor
sudo supervisorctl reread
sudo supervisorctl update
sudo supervisorctl restart all

# Production (systemd)
bench setup procfile
pm2 start pm2.config.js  # or use systemd
```

---

## 3) Chunking Strategy (Critical)

- **Chunk size**: 100–500 items per job (tune by payload and DB cost)
- **Rationale**: limits blast radius, enables retries, uses parallelism
- **Ordering**: stable slices to avoid duplicates

Example slicing: see the TL;DR `enqueue_bulk` code.

---

## 4) Idempotency & Safety

- **Idempotent task**: running the same job twice shouldn't corrupt data
- Use **per-record savepoints** for isolation
- Consider a **status field** (e.g., `processing`, `done`, `error`) on the target doctype to avoid rework
- Add a **dedupe key** if external side-effects exist

**Safe error handling pattern:**

```python
def process_records(ids: list[str]):
    frappe.db.commit()  # Clean start

    for docname in ids:
        try:
            savepoint = f"bulk_{docname}"
            frappe.db.savepoint(savepoint)

            # Your processing logic
            doc = frappe.get_doc("ToDo", docname)
            # ... work ...
            doc.save()

        except Exception as e:
            frappe.db.rollback(save_point=savepoint)
            frappe.log_error(f"Failed {docname}: {str(e)}", "bulk.process_records")

    frappe.db.commit()  # Commit successes
```

---

## 5) Timeouts, Retries, Failures

- **Timeout** per job via `timeout=...` seconds
- **Auto-retry**: Frappe/RQ does not auto-retry by default. Implement retry logic:

```python
import time

def with_retry(operation, max_attempts=3, delay=1):
    for attempt in range(1, max_attempts + 1):
        try:
            return operation()
        except Exception as e:
            if attempt == max_attempts:
                raise
            time.sleep(delay * attempt)  # Exponential backoff

# Usage in your task
def process_records(ids: list[str]):
    for docname in ids:
        def process_single():
            # Your processing logic
            doc = frappe.get_doc("ToDo", docname)
            doc.save()

        with_retry(process_single, max_attempts=3)
```

---

## 6) Observability (See What's Happening)

### Monitor Jobs - CORRECTED COMMANDS

```bash
# ✅ CORRECT: Show pending jobs and worker status
bench --site yoursite doctor

# ✅ CORRECT: Show background job queues
# Visit: Desk → Background Jobs (in Frappe UI)

# ✅ CORRECT: Purge jobs (be careful!)
bench --site yoursite purge-jobs

# ✅ Check worker logs
tail -f sites/yoursite/logs/worker.log
tail -f sites/yoursite/logs/worker.error.log

# ✅ Check redis queue status
redis-cli  # then use Redis commands
KEYS rq:*
LLEN rq:queue:default  # Check queue length
```

### Logs & UI

- **Logs**: `sites/yoursite/logs/worker.error.log` and `worker.log`
- **Background Jobs page**: Awesome Bar → "Background Jobs"
- **Real-time progress** (optional):

```python
from frappe import publish_realtime

def process_records(ids: list[str]):
    total = len(ids)
    for i, docname in enumerate(ids):
        # Process document
        if i % 10 == 0:  # Update every 10 records
            publish_realtime("bulk_progress", {
                "progress": f"{i+1}/{total}",
                "docname": docname
            })
```

---

## 7) Client-Side Patterns

### List View Bulk Action

```javascript
// Add to List View settings
frappe.listview_settings["ToDo"].get_indicator = function (doc) {
  // Your indicator logic
};

// Custom button in List View
frappe.listview_settings["ToDo"].onload = function (listview) {
  listview.page.add_action_item(__("Bulk Process"), function () {
    let selected = listview.get_checked_items();
    frappe.call({
      method: "your_app.api.bulk.enqueue_bulk",
      args: { ids: selected.map((d) => d.name) },
      callback: function (r) {
        frappe.show_alert(__("Enqueued {0} jobs", [r.message.enqueued]));
      },
    });
  });
};
```

### Doctype Button

```python
# In your doctype.py
def bulk_process(self):
    frappe.call("your_app.api.bulk.enqueue_bulk",
        ids=[self.name],  # or filtered list
        queue="long"
    )
```

---

## 8) Site & Permissions

- Jobs run in the **site context** active at enqueue time
- If you enqueue cross-site, pass `site=...` in `frappe.enqueue`
- Permissions are **not enforced** in background jobs. Add explicit checks:

```python
def process_records(ids: list[str]):
    for docname in ids:
        # Verify document exists and is accessible
        if not frappe.db.exists("ToDo", docname):
            frappe.log_error(f"Document {docname} not found", "bulk.process_records")
            continue
        # Your processing logic
```

---

## 9) Common Recipes

### (A) Map over filtered docs

```python
# Get filtered document IDs
ids = frappe.get_all("ToDo",
    filters={"status": "Open"},
    pluck="name"
)
frappe.enqueue("your_app.tasks.bulk.process_records", ids=ids)
```

### (B) Heavy external API calls

```python
def process_with_api(ids: list[str]):
    for docname in ids:
        # Long-running API call with retry
        response = with_retry(lambda: call_external_api(docname), max_attempts=3)
        # Process response
```

### (C) Batch updates with db_set

```python
# Faster than doc.save() for single field updates
def bulk_status_update(ids: list[str], new_status: str):
    for docname in ids:
        frappe.db.set_value("ToDo", docname, "status", new_status)
```

---

## 10) Cancel / Re-queue

### Cancel Jobs - CORRECTED COMMANDS

```bash
bench --site yoursite purge-jobs

# ✅ From Python console
frappe.get_doc("RQ Job", "job_id").delete()

# ✅ Using Frappe UI: Desk → Background Jobs → Cancel jobs
```

### Re-queue Failed Chunks

```python
# Extract failed IDs from logs and re-queue
failed_ids = [...]  # From error logs
frappe.enqueue("your_app.tasks.bulk.process_records", ids=failed_ids)
```

---

## 11) Performance Tips

- Use **`doc.db_set`** instead of `doc.save()` for single-field updates
- **Pre-fetch data** to avoid N+1 queries
- Keep tasks **import-light** - import at module level, not in loops
- **Batch DB operations** where safe:

```python
def efficient_bulk_update(ids: list[str]):
    updates = []
    for docname in ids:
        # Collect updates
        updates.append((docname, {"status": "Processed"}))

    # Batch update
    frappe.db.bulk_update("ToDo", updates)
```

---

## 12) Security & Data Integrity

- Validate input IDs exist and belong to correct site
- Sanitize args (limit `chunk_size` ≤ 1000)
- Don't pass huge payloads via args; pass IDs and fetch server-side
- Add rate limiting if processing user-generated content

```python
@frappe.whitelist()
def enqueue_bulk(ids: list[str] | str, chunk_size: int = 200, queue: str = "long"):
    # Validate inputs
    if chunk_size > 1000:
        frappe.throw("Chunk size too large")

    if isinstance(ids, str):
        ids = [x.strip() for x in ids.split(",") if x.strip()]

    # Limit maximum records
    if len(ids) > 10000:
        frappe.throw("Too many records")

    # Continue with enqueue logic...
```

---

## 13) Testing

### Test in Console - CORRECTED COMMANDS

```bash
bench --site yoursite console
```

```python
# Test enqueue
from your_app.api.bulk import enqueue_bulk
ids = [d.name for d in frappe.get_all("ToDo", limit=100)]
result = enqueue_bulk(ids, chunk_size=50, queue="short")
print(f"Enqueued {result['enqueued']} jobs")

# Check queue status using doctor
from frappe.utils.background_jobs import get_queues
queues = get_queues()
print(f"Default queue length: {queues['default'].count}")

# Or check via UI
print("Visit Desk → Background Jobs to see queued jobs")
```

### Unit Test

```python
import frappe
from your_app.tasks.bulk import process_records

def test_bulk_processing():
    # Create test records
    test_docs = []
    for i in range(5):
        doc = frappe.get_doc({"doctype": "ToDo", "description": f"Test {i}"})
        doc.insert()
        test_docs.append(doc.name)

    # Test processing
    process_records(test_docs)

    # Verify results
    for name in test_docs:
        doc = frappe.get_doc("ToDo", name)
        assert doc.status == "Processed"  # Your expected outcome
```

---

## 14) FAQ

**Which queue to choose?**

- `short` (<10s), `default` (<2m), `long` (>2m)

**Where do jobs show up?**

- Desk → Background Jobs (Frappe UI)

**How to monitor queue status?**

```bash
bench --site yoursite doctor          # Worker status
bench --site yoursite show-pending-jobs  # Pending jobs
```

**Do jobs keep user permissions?**

- No. They run server-side. Validate explicitly if needed

**Can I schedule for later execution?**

```python
# Execute after commit
frappe.enqueue(
    "your_app.tasks.bulk.process_records",
    enqueue_after_commit=True,
    at_front=False,
    job_id=job_id,
    queue="long",
    timeout=3600,
    ids=chunk
)
```

**How to handle very large datasets (>100K records)?**

- Use generator patterns to avoid memory issues
- Process in smaller chunks (50-100 records)
- Consider using dedicated long-running workers

---

## 15) Minimal Boilerplate Structure

```
your_app/
├── __init__.py
├── api/
│   ├── __init__.py
│   └── bulk.py          # enqueue_bulk function
└── tasks/
    ├── __init__.py
    └── bulk.py          # process_records function
```

That's it. Start workers, enqueue in chunks, keep tasks idempotent, monitor from **Desk → Background Jobs** or using `bench doctor` and `bench show-pending-jobs`. Done.
