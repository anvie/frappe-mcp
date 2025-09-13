# Scheduler & Background Jobs in Frappe

In Frappe, there are **two main types of background execution**:

1. **Scheduled Jobs (Scheduler Events)**
   Declared in `hooks.py`. They are automatically executed by the _scheduler_ at a specific interval (minute, hourly, daily, weekly, monthly) or by a custom **cron** pattern.

2. **Background Jobs (Enqueue/RQ)**
   Triggered on demand using `frappe.enqueue(...)` or `frappe.enqueue_doc(...)`. These are queued tasks, ideal for long-running processes so the UI stays responsive.

Both rely on **Redis** + **RQ workers**, managed per-site.

---

## Requirements & Concepts

- **Redis** must be running (for queues and cache).
- Start bench with workers & scheduler:

  ```bash
  bench start
  # or in production (using supervisor/systemd)
  # supervisorctl start all
  ```

- Enable the scheduler for a site:

  ```bash
  bench --site <site-name> enable-scheduler
  bench --site <site-name> doctor  # check status
  ```

- Disable scheduler if needed:

  ```bash
  bench --site <site-name> disable-scheduler
  ```

- **Timezone**: Scheduler runs based on the server’s timezone (unless you configure it on the site).

---

## Scheduled Jobs via `hooks.py`

In your app’s `hooks.py`, add `scheduler_events`.

### Built-in intervals

```python
scheduler_events = {
    "all": [
        "my_app.api.sync_quick_tasks",     # runs every minute tick
    ],
    "hourly": [
        "my_app.api.rebuild_hourly_reports",
    ],
    "daily": [
        "my_app.api.rollover_daily_settlements",
    ],
    "weekly": [
        "my_app.api.send_weekly_digest",
    ],
    "monthly": [
        "my_app.api.close_month_end",
    ],
}
```

Each entry is a list of Python functions (dot-path). They should not require arguments.

### Cron patterns

Use standard cron format: `min hour day month day_of_week`.

```python
scheduler_events = {
    "cron": {
        "* * * * *": ["my_app.api.healthcheck"],         # every minute
        "30 1 * * *": ["my_app.api.run_nightly_jobs"],   # daily at 01:30
        "0 2 * * 1": ["my_app.api.billing_weekly"],      # Mondays at 02:00
        "0 3 28-31 * *": ["my_app.api.try_close_month"], # last-day trick
    }
}
```

---

# 4) Background Jobs (RQ)

Use background jobs to queue tasks triggered by user actions or system events.

### `frappe.enqueue`

```python
import frappe

def heavy_task(record_id: str):
    # ... long-running work ...
    pass

def controller():
    frappe.enqueue(
        "my_app.api.heavy_task",
        queue="long",              # short | default | long
        job_name="heavy-task",
        timeout=60 * 30,           # seconds
        enqueue_after_commit=True, # run only after DB commit
        kwargs={"record_id": "ABC123"},
    )
```

### Important parameters:

- `queue`:

  - **short**: < 20s
  - **default**: normal
  - **long**: heavy jobs

- `timeout`: job timeout in seconds.
- `enqueue_after_commit=True`: ensures it runs after DB changes are committed.

### `frappe.enqueue_doc`

Queue a method on a document.

```python
frappe.enqueue_doc(
    doctype="Sales Invoice",
    name="SINV-0001",
    method="generate_einvoice_and_send",
    queue="long",
    timeout=1800,
    send_email=True
)
```

In the DocType class:

```python
class SalesInvoice(Document):
    def generate_einvoice_and_send(self, send_email=False):
        # access self.<field>
        # business logic
        if send_email:
            self.notify_update()
```

---

# 5) Running Workers & Queues

### Development

```bash
bench start
```

This runs web, scheduler, and workers (`short`, `default`, `long`).

### Production

In your **Procfile**:

```
schedule: bench schedule
worker: bench worker
short: bench worker --queue short
long: bench worker --queue long
```

Run them via **supervisor** or **systemd**.

### Monitoring

- Check site health:

  ```bash
  bench --site <site> doctor
  ```

- View pending jobs:

  ```bash
  bench --site <site> show-queue
  bench --site <site> show-pending-jobs
  ```

- Force clear jobs:

  ```bash
  bench --site <site> purge-jobs
  ```

---

## Best Practices

- Use queues based on job size (`short`, `default`, `long`).
- Make jobs **idempotent** and safe for retries.
- Pass IDs, not big objects, in `kwargs`.
- Set realistic timeouts.
- Use `frappe.logger()` for logging.
- Handle API retries with backoff.
- Validate inputs for security.

---

## End-to-End Example

### Scheduled jobs

`my_app/hooks.py`:

```python
scheduler_events = {
    "daily": ["my_app.jobs.daily.rollover"],
    "cron": {"15 0 * * *": ["my_app.jobs.daily.cleanup_temp"]},
}
```

`my_app/jobs/daily.py`:

```python
import frappe

def rollover():
    frappe.logger().info("Daily rollover running")
    # business logic

def cleanup_temp():
    # delete old temp files
    pass
```

### Background job with API

`my_app/api.py`:

```python
import frappe

@frappe.whitelist()
def start_heavy_job(record_id: str):
    frappe.enqueue(
        "my_app.jobs.bg.build_big_report",
        queue="long",
        timeout=3600,
        enqueue_after_commit=True,
        kwargs={"record_id": record_id},
    )
    return {"status": "queued"}
```

`my_app/jobs/bg.py`:

```python
def build_big_report(record_id: str):
    # heavy computation
    pass
```

Client call:

```javascript
frappe.call({
  method: "my_app.api.start_heavy_job",
  args: { record_id: cur_frm.doc.name },
  callback: () => frappe.show_alert("Report queued"),
});
```

---

## Debugging & Testing

- Run function manually:

  ```bash
  bench --site <site> execute my_app.jobs.daily.rollover
  ```

- Trigger scheduler event:

  ```bash
  bench --site <site> trigger-scheduler-event daily
  ```

- Test enqueue:

  ```bash
  bench --site <site> execute my_app.api.start_heavy_job --kwargs "{'record_id':'TEST-001'}"
  ```

---

## Quick FAQ

**Why is my job not running?**
Check (1) scheduler enabled, (2) workers running, (3) function path valid.

**When to use scheduled vs enqueue?**

- Scheduled: recurring (reports, backups).
- Enqueue: on-demand (submit doc, export report).

**How to separate SLAs?**
Use multiple queues + supervisor with different worker counts.

---

## Quick Checklist

1. Write business function.
2. Register in `hooks.py` (scheduler) or call via `enqueue`.
3. Ensure workers/scheduler are running.
4. Add logging & error handling.
5. Test with `bench execute`.
6. Monitor with `bench doctor` and `show-queue`.

---

Do you also want me to create a **step-by-step supervisor config template** (for production) alongside this?
