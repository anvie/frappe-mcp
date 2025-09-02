# Sample Python file for testing code snippet extraction
# Line 2
def process_task(task_id):
    """Process a task by ID"""
    # Line 5 - Get the task document
    task = frappe.get_doc("Task", task_id)
    
    # Check task status
    if task.status == "Open":
        # Line 10
        print(f"Processing task: {task.name}")
        task.status = "In Progress"
        task.save()
        
    # Line 15 - Check for completion
    elif task.status == "In Progress":
        # Do some work
        result = perform_task_work(task)
        
        # Line 20
        if result.success:
            task.status = "Completed"
            task.completion_date = now()
            task.save()
        else:
            # Line 26
            task.status = "Failed"
            task.error_message = result.error
            task.save()
    
    # Line 31
    return task


def perform_task_work(task):
    """Simulate task work"""
    # Line 37
    import random
    
    # Line 40 - Random success/failure
    success = random.choice([True, False])
    
    class Result:
        def __init__(self, success, error=None):
            # Line 45
            self.success = success
            self.error = error
    
    if success:
        # Line 50
        return Result(True)
    else:
        return Result(False, "Random failure for testing")