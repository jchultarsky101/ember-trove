// Node-type Markdown templates.
//
// Applied once when a new node is created; the user owns the body afterwards.
// Templates omit a To-Do section — structured tasks are managed in the task
// panel on the node view, not in free-form Markdown.

pub fn template_for_type(node_type: &str) -> &'static str {
    match node_type {
        "project" => PROJECT,
        "area" => AREA,
        "resource" => RESOURCE,
        "reference" => REFERENCE,
        _ => "",
    }
}

const PROJECT: &str = "\
## Status

- Active

## Goals

-

## Team

| Name | Function | Email | Comment |
|------|----------|-------|---------|
|      |          |       |         |
";

const AREA: &str = "\
## Purpose

Describe the scope and purpose of this area.

## Resources

-
";

const RESOURCE: &str = "\
## Summary

Brief description of this resource.

## Links

-
";

const REFERENCE: &str = "\
## Citation

>

## Notes

-
";
