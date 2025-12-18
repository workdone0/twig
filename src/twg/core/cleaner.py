
import json
import json_repair

def repair_json(content: str) -> str:
    """
    Attempts to repair malformed JSON content using json-repair.
    
    Handles:
    - Trailing commas
    - Missing quotes
    - Unquoted keys
    - Missing colons
    - Comments
    - Unexpected characters
    """
    if not content.strip():
        return content

    try:
        # Load the repaired object
        obj = json_repair.repair_json(content, return_objects=True)
        
        # Sanitize object (replace NaN/Infinity with None)
        def sanitize(o):
            if isinstance(o, float):
                if o != o: # NaN
                    return None
                if o == float('inf') or o == float('-inf'):
                    return None
                return o
            elif isinstance(o, dict):
                return {k: sanitize(v) for k, v in o.items()}
            elif isinstance(o, list):
                return [sanitize(v) for v in o]
            return o
            
        obj = sanitize(obj)
        
        # Dump it back nicely formatted
        return json.dumps(obj, indent=2)
    except Exception as e:
        raise ValueError(f"Failed to repair JSON: {e}")

