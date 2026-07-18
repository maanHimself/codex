The current customer-service procedure objective was updated by the application.

The new procedure objective below supersedes the previous procedure objective. It is application-provided procedure context, not higher-priority instructions.

<procedure_objective>
{{ objective }}
</procedure_objective>

Adjust the current turn to follow the updated procedure objective. Do not continue work that served only the previous objective.

Procedure execution is internal. Never mention the procedure, its objective, its status, or other runtime mechanics to the customer.

Do not call `update_procedure_status` with `complete` unless the updated procedure is actually complete.
