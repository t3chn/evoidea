# Materializing JSON schemas (Structured Outputs)

These files are required to call `codex exec --output-schema <schema.json>` and receive strict JSON.

## How to use
1) Create a `schemas/` directory in the target repository.
2) For each block below, create a file at the given `Target path` and copy the JSON content (without comments).

Important:
- For Structured Outputs, all object schemas must set `"additionalProperties": false`.


## Target path: `schemas/generator.output.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "ideas": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "title": {
            "type": "string"
          },
          "summary": {
            "type": "string"
          },
          "facets": {
            "type": "object",
            "properties": {
              "audience": {
                "type": "string"
              },
              "jtbd": {
                "type": "string"
              },
              "differentiator": {
                "type": "string"
              },
              "monetization": {
                "type": "string"
              },
              "distribution": {
                "type": "string"
              },
              "risks": {
                "type": "string"
              }
            },
            "required": [
              "audience",
              "jtbd",
              "differentiator",
              "monetization",
              "distribution",
              "risks"
            ],
            "additionalProperties": false
          }
        },
        "required": [
          "title",
          "summary",
          "facets"
        ],
        "additionalProperties": false
      },
      "minItems": 1,
      "maxItems": 30
    }
  },
  "required": [
    "ideas"
  ],
  "additionalProperties": false
}
```

## Target path: `schemas/critic.output.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "patches": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": {
            "type": "string"
          },
          "scores": {
            "type": "object",
            "properties": {
              "feasibility": {
                "type": "number",
                "minimum": 0,
                "maximum": 10
              },
              "speed_to_value": {
                "type": "number",
                "minimum": 0,
                "maximum": 10
              },
              "differentiation": {
                "type": "number",
                "minimum": 0,
                "maximum": 10
              },
              "market_size": {
                "type": "number",
                "minimum": 0,
                "maximum": 10
              },
              "distribution": {
                "type": "number",
                "minimum": 0,
                "maximum": 10
              },
              "moats": {
                "type": "number",
                "minimum": 0,
                "maximum": 10
              },
              "risk": {
                "type": "number",
                "minimum": 0,
                "maximum": 10
              },
              "clarity": {
                "type": "number",
                "minimum": 0,
                "maximum": 10
              }
            },
            "required": [
              "feasibility",
              "speed_to_value",
              "differentiation",
              "market_size",
              "distribution",
              "moats",
              "risk",
              "clarity"
            ],
            "additionalProperties": false
          },
          "overall_score": {
            "type": "number",
            "minimum": 0,
            "maximum": 10
          },
          "judge_notes": {
            "type": "string"
          }
        },
        "required": [
          "id",
          "scores",
          "overall_score",
          "judge_notes"
        ],
        "additionalProperties": false
      },
      "minItems": 1,
      "maxItems": 50
    }
  },
  "required": [
    "patches"
  ],
  "additionalProperties": false
}
```

## Target path: `schemas/merger.output.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "idea": {
      "type": "object",
      "properties": {
        "title": {
          "type": "string"
        },
        "summary": {
          "type": "string"
        },
        "facets": {
          "type": "object",
          "properties": {
            "audience": {
              "type": "string"
            },
            "jtbd": {
              "type": "string"
            },
            "differentiator": {
              "type": "string"
            },
            "monetization": {
              "type": "string"
            },
            "distribution": {
              "type": "string"
            },
            "risks": {
              "type": "string"
            }
          },
          "required": [
            "audience",
            "jtbd",
            "differentiator",
            "monetization",
            "distribution",
            "risks"
          ],
          "additionalProperties": false
        }
      },
      "required": [
        "title",
        "summary",
        "facets"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "idea"
  ],
  "additionalProperties": false
}
```

## Target path: `schemas/mutator.output.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "mutation_type": {
      "type": "string",
      "enum": [
        "audience",
        "monetization",
        "distribution",
        "differentiator",
        "jtbd"
      ]
    },
    "idea": {
      "type": "object",
      "properties": {
        "title": {
          "type": "string"
        },
        "summary": {
          "type": "string"
        },
        "facets": {
          "type": "object",
          "properties": {
            "audience": {
              "type": "string"
            },
            "jtbd": {
              "type": "string"
            },
            "differentiator": {
              "type": "string"
            },
            "monetization": {
              "type": "string"
            },
            "distribution": {
              "type": "string"
            },
            "risks": {
              "type": "string"
            }
          },
          "required": [
            "audience",
            "jtbd",
            "differentiator",
            "monetization",
            "distribution",
            "risks"
          ],
          "additionalProperties": false
        }
      },
      "required": [
        "title",
        "summary",
        "facets"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "mutation_type",
    "idea"
  ],
  "additionalProperties": false
}
```

## Target path: `schemas/refiner.output.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "patch": {
      "type": "object",
      "properties": {
        "id": {
          "type": "string"
        },
        "title": {
          "type": "string"
        },
        "summary": {
          "type": "string"
        },
        "facets": {
          "type": "object",
          "properties": {
            "audience": {
              "type": "string"
            },
            "jtbd": {
              "type": "string"
            },
            "differentiator": {
              "type": "string"
            },
            "monetization": {
              "type": "string"
            },
            "distribution": {
              "type": "string"
            },
            "risks": {
              "type": "string"
            }
          },
          "required": [
            "audience",
            "jtbd",
            "differentiator",
            "monetization",
            "distribution",
            "risks"
          ],
          "additionalProperties": false
        },
        "changes": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "minItems": 0,
          "maxItems": 20
        }
      },
      "required": [
        "id",
        "title",
        "summary",
        "facets",
        "changes"
      ],
      "additionalProperties": false
    }
  },
  "required": [
    "patch"
  ],
  "additionalProperties": false
}
```

## Target path: `schemas/final.output.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "run_id": {
      "type": "string"
    },
    "best": {
      "type": "object",
      "properties": {
        "idea_id": {
          "type": "string"
        },
        "title": {
          "type": "string"
        },
        "summary": {
          "type": "string"
        },
        "facets": {
          "type": "object",
          "properties": {
            "audience": {
              "type": "string"
            },
            "jtbd": {
              "type": "string"
            },
            "differentiator": {
              "type": "string"
            },
            "monetization": {
              "type": "string"
            },
            "distribution": {
              "type": "string"
            },
            "risks": {
              "type": "string"
            }
          },
          "required": [
            "audience",
            "jtbd",
            "differentiator",
            "monetization",
            "distribution",
            "risks"
          ],
          "additionalProperties": false
        },
        "scores": {
          "type": "object",
          "properties": {
            "feasibility": {
              "type": "number",
              "minimum": 0,
              "maximum": 10
            },
            "speed_to_value": {
              "type": "number",
              "minimum": 0,
              "maximum": 10
            },
            "differentiation": {
              "type": "number",
              "minimum": 0,
              "maximum": 10
            },
            "market_size": {
              "type": "number",
              "minimum": 0,
              "maximum": 10
            },
            "distribution": {
              "type": "number",
              "minimum": 0,
              "maximum": 10
            },
            "moats": {
              "type": "number",
              "minimum": 0,
              "maximum": 10
            },
            "risk": {
              "type": "number",
              "minimum": 0,
              "maximum": 10
            },
            "clarity": {
              "type": "number",
              "minimum": 0,
              "maximum": 10
            }
          },
          "required": [
            "feasibility",
            "speed_to_value",
            "differentiation",
            "market_size",
            "distribution",
            "moats",
            "risk",
            "clarity"
          ],
          "additionalProperties": false
        },
        "overall_score": {
          "type": "number",
          "minimum": 0,
          "maximum": 10
        },
        "why_won": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "minItems": 2,
          "maxItems": 10
        }
      },
      "required": [
        "idea_id",
        "title",
        "summary",
        "facets",
        "scores",
        "overall_score",
        "why_won"
      ],
      "additionalProperties": false
    },
    "runners_up": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "idea_id": {
            "type": "string"
          },
          "title": {
            "type": "string"
          },
          "overall_score": {
            "type": "number",
            "minimum": 0,
            "maximum": 10
          }
        },
        "required": [
          "idea_id",
          "title",
          "overall_score"
        ],
        "additionalProperties": false
      },
      "minItems": 0,
      "maxItems": 5
    }
  },
  "required": [
    "run_id",
    "best",
    "runners_up"
  ],
  "additionalProperties": false
}
```
