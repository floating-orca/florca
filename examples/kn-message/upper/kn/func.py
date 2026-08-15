def handle(request_body: dict) -> dict:
    input = request_body["payload"]
    return {
        "payload": input.upper(),
        "next": None,
    }


def main(c):
    try:
        return handle(c.request.get_json()), 200
    except Exception as e:
        return {"error": f"{type(e).__name__}: {e}"}, 500
