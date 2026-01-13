const { sendMessage, sendMessageToParent, sendMessageToWorkflow } = require("./fn.js");

exports.handler = async (requestBody) => {
  const input = requestBody.payload;
  const context = requestBody.context;
  return {
    payload: {},
    next: undefined,
  };
};
