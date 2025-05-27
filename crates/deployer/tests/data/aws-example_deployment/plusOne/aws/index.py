def handler(requestBody, _):
    return {
        "payload": requestBody["payload"] + 1,
        "next": "double",
    }
