# policy

*Source: `/Users/g.bogans/Downloads/DEK_in_FARD/packages/kernel-policy/src/policy.fard`*

---

## `make_policy(allowed_programs, allowed_oracles, allowed_exits, max_step_count, expires_at)`

*Line 21*

## `policy_cid(policy)`

*Line 42*

## `revoke(policy)`

*Line 46*

## `is_revoked(policy)`

*Line 56*

## `get_field(policy, key)`

*Line 65*

## `is_option_none(v)`

*Line 71*

## `list_contains_text(lst, target)`

*Line 75*

## `check_program(policy, program_cid)`

*Line 81*

## `check_oracle(policy, oracle_kind)`

*Line 87*

## `check_exit(policy, exit_code)`

*Line 93*

## `check_expiry(policy, now_ms)`

*Line 99*

## `check_step_count(policy, steps)`

*Line 106*

## `enforce(policy, program_cid, exit_code, oracle_kinds, step_count, now_ms)`

*Line 118*

