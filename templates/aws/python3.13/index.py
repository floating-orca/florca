from fn import send_message, send_message_to_parent, send_message_to_workflow


def handler(request_body: dict, _) -> dict:
    input = request_body["payload"]
    context: dict = request_body["context"]
    return {
        "payload": {},
        "next": None,
    }
