# A/B Testing API Documentation

## Base URL
```
http://localhost:8082/api/v1
```

## Endpoints

### 1. Create Experiment

**POST** `/experiments`

Create a new A/B test experiment with multiple variants.

#### Request Body
```json
{
  "name": "lora_boost_test",
  "description": "Test LoRA boost factor 0.3 vs 0.5",
  "traffic_allocation": 0.5,
  "variants": [
    {
      "name": "control",
      "weight": 0.5,
      "config": {
        "lora_boost": 0.3,
        "algorithm": "baseline"
      }
    },
    {
      "name": "treatment",
      "weight": 0.5,
      "config": {
        "lora_boost": 0.5,
        "algorithm": "enhanced"
      }
    }
  ]
}
```

#### Response (200 OK)
```json
{
  "experiment_id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "lora_boost_test",
  "status": "draft",
  "variants": 2
}
```

#### Validation Rules
- Minimum 2 variants required
- Variant weights must sum to 1.0
- Traffic allocation must be 0.0-1.0
- Experiment name must be unique

---

### 2. Get Experiment

**GET** `/experiments/{experiment_id}`

Retrieve complete experiment details including all variants.

#### Path Parameters
- `experiment_id` (UUID) - Experiment identifier

#### Response (200 OK)
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "lora_boost_test",
  "description": "Test LoRA boost factor 0.3 vs 0.5",
  "status": "running",
  "traffic_allocation": 0.5,
  "created_at": "2025-12-06T12:00:00Z",
  "updated_at": "2025-12-06T12:05:00Z",
  "started_at": "2025-12-06T12:05:00Z",
  "completed_at": null,
  "variants": [
    {
      "id": "650e8400-e29b-41d4-a716-446655440001",
      "experiment_id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "control",
      "weight": 0.5,
      "config": {
        "lora_boost": 0.3,
        "algorithm": "baseline"
      },
      "created_at": "2025-12-06T12:00:00Z"
    },
    {
      "id": "650e8400-e29b-41d4-a716-446655440002",
      "experiment_id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "treatment",
      "weight": 0.5,
      "config": {
        "lora_boost": 0.5,
        "algorithm": "enhanced"
      },
      "created_at": "2025-12-06T12:00:00Z"
    }
  ]
}
```

#### Response (404 Not Found)
```json
{
  "error": "Experiment not found",
  "message": "Experiment not found: 550e8400-e29b-41d4-a716-446655440000"
}
```

---

### 3. List Experiments

**GET** `/experiments`

List all currently running experiments.

#### Response (200 OK)
```json
{
  "experiments": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "lora_boost_test",
      "description": "Test LoRA boost factor 0.3 vs 0.5",
      "status": "running",
      "traffic_allocation": 0.5,
      "created_at": "2025-12-06T12:00:00Z",
      "updated_at": "2025-12-06T12:05:00Z",
      "started_at": "2025-12-06T12:05:00Z",
      "completed_at": null,
      "variants": [...]
    }
  ],
  "count": 1
}
```

---

### 4. Update Experiment Status

**PUT** `/experiments/{experiment_id}/status`

Change experiment status (draft → running → completed).

#### Path Parameters
- `experiment_id` (UUID) - Experiment identifier

#### Request Body
```json
{
  "status": "running"
}
```

#### Valid Status Values
- `draft` - Experiment configured but not started
- `running` - Experiment active and assigning users
- `paused` - Experiment temporarily stopped
- `completed` - Experiment finished

#### Response (200 OK)
```json
{
  "experiment_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "running"
}
```

#### Response (400 Bad Request)
```json
{
  "error": "Invalid status",
  "valid_statuses": ["draft", "running", "paused", "completed"]
}
```

---

### 5. Get Experiment Metrics

**GET** `/experiments/{experiment_id}/metrics`

Retrieve aggregated metrics for all variants in an experiment.

#### Path Parameters
- `experiment_id` (UUID) - Experiment identifier

#### Response (200 OK)
```json
{
  "experiment_id": "550e8400-e29b-41d4-a716-446655440000",
  "variant_metrics": {
    "650e8400-e29b-41d4-a716-446655440001": {
      "variant_id": "650e8400-e29b-41d4-a716-446655440001",
      "variant_name": "control",
      "exposures": 1250,
      "unique_users": 1000,
      "conversions": {
        "watch_completion": {
          "count": 450,
          "sum": 450.0,
          "mean": 1.0,
          "conversion_rate": 0.36
        },
        "click_through": {
          "count": 312,
          "sum": 312.0,
          "mean": 1.0,
          "conversion_rate": 0.2496
        }
      }
    },
    "650e8400-e29b-41d4-a716-446655440002": {
      "variant_id": "650e8400-e29b-41d4-a716-446655440002",
      "variant_name": "treatment",
      "exposures": 1275,
      "unique_users": 1015,
      "conversions": {
        "watch_completion": {
          "count": 485,
          "sum": 485.0,
          "mean": 1.0,
          "conversion_rate": 0.38
        },
        "click_through": {
          "count": 340,
          "sum": 340.0,
          "mean": 1.0,
          "conversion_rate": 0.2667
        }
      }
    }
  },
  "computed_at": "2025-12-06T13:00:00Z"
}
```

#### Metric Fields
- `exposures` - Total number of times variant was shown
- `unique_users` - Number of distinct users who saw variant
- `conversions[metric_name]`
  - `count` - Number of conversion events
  - `sum` - Total conversion value
  - `mean` - Average conversion value
  - `conversion_rate` - Conversions divided by exposures

---

### 6. Record Conversion

**POST** `/experiments/conversions`

Record a conversion event (goal completion) for a user.

#### Request Body
```json
{
  "experiment_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "750e8400-e29b-41d4-a716-446655440000",
  "metric_name": "watch_completion",
  "value": 1.0,
  "metadata": {
    "duration_seconds": 3600,
    "content_id": "850e8400-e29b-41d4-a716-446655440000"
  }
}
```

#### Response (200 OK)
```json
{
  "status": "recorded",
  "experiment_id": "550e8400-e29b-41d4-a716-446655440000",
  "variant": "treatment",
  "metric": "watch_completion",
  "value": 1.0
}
```

#### Response (400 Bad Request)
```json
{
  "error": "User not in experiment"
}
```

#### Common Metric Names
- `watch_completion` - User finished watching content
- `click_through` - User clicked on recommendation
- `engagement_time` - Time spent with content
- `rating_given` - User provided rating
- `add_to_watchlist` - User saved content

---

## Integration with Recommendations

When requesting recommendations, the system automatically:
1. Checks for running experiments
2. Assigns user to variant (if not already assigned)
3. Records exposure event
4. Adds `experiment_variant` field to response

### Enhanced Recommendation Response

**POST** `/recommendations`

```json
{
  "user_id": "750e8400-e29b-41d4-a716-446655440000",
  "limit": 10
}
```

**Response includes experiment info:**
```json
{
  "recommendations": [
    {
      "content_id": "950e8400-e29b-41d4-a716-446655440000",
      "confidence_score": 0.87,
      "recommendation_type": "Hybrid",
      "based_on": ["genre_match", "user_history"],
      "explanation": "Based on your recent viewing history"
    }
  ],
  "generated_at": "2025-12-06T13:00:00Z",
  "ttl_seconds": 3600
}
```

*Note: The `experiment_variant` field is set internally but may not be visible in recommendation DTOs. Check variant assignment separately via the experiments API.*

---

## Workflow Example

### Complete A/B Test Lifecycle

```bash
# 1. Create experiment
curl -X POST http://localhost:8082/api/v1/experiments \
  -H "Content-Type: application/json" \
  -d '{
    "name": "recommendation_algo_test",
    "description": "Test new recommendation algorithm",
    "traffic_allocation": 1.0,
    "variants": [
      {"name": "baseline", "weight": 0.5, "config": {"algo": "v1"}},
      {"name": "new_algo", "weight": 0.5, "config": {"algo": "v2"}}
    ]
  }'

# Response: {"experiment_id": "..."}

# 2. Start experiment
curl -X PUT http://localhost:8082/api/v1/experiments/{id}/status \
  -H "Content-Type: application/json" \
  -d '{"status": "running"}'

# 3. Users get recommendations (automatic assignment and exposure tracking)
curl -X POST http://localhost:8082/api/v1/recommendations \
  -H "Content-Type: application/json" \
  -d '{"user_id": "..."}'

# 4. Record conversions when users watch content
curl -X POST http://localhost:8082/api/v1/experiments/conversions \
  -H "Content-Type: application/json" \
  -d '{
    "experiment_id": "...",
    "user_id": "...",
    "metric_name": "watch_completion",
    "value": 1.0
  }'

# 5. Check metrics
curl http://localhost:8082/api/v1/experiments/{id}/metrics

# 6. Complete experiment
curl -X PUT http://localhost:8082/api/v1/experiments/{id}/status \
  -H "Content-Type: application/json" \
  -d '{"status": "completed"}'
```

---

## Error Responses

### 400 Bad Request
- Invalid experiment configuration
- Variant weights don't sum to 1.0
- Invalid status transition
- User not in experiment

### 404 Not Found
- Experiment ID doesn't exist

### 500 Internal Server Error
- Database connection failure
- Serialization error
- Unexpected error

---

## Best Practices

### 1. Experiment Design
- Define clear success metrics before starting
- Ensure sufficient sample size
- Run for appropriate duration (1-2 weeks minimum)
- Monitor for sample ratio mismatch (SRM)

### 2. Variant Configuration
- Keep control group stable
- Test one variable at a time
- Document configuration changes
- Use semantic versioning for configs

### 3. Metrics Tracking
- Track multiple metrics (primary + secondary)
- Include guardrail metrics
- Record metadata for debugging
- Monitor exposure/conversion ratios

### 4. Statistical Rigor
- Calculate required sample size beforehand
- Don't peek at results too early
- Use proper significance testing
- Account for multiple comparisons

---

## Database Schema

See `/workspaces/media-gateway/migrations/ab_testing_schema.sql` for complete schema.

Tables:
- `experiments` - Experiment configuration
- `experiment_variants` - Variant definitions
- `experiment_assignments` - User assignments
- `experiment_exposures` - Impression tracking
- `experiment_conversions` - Goal completions

---

## Further Reading

- [BATCH_004_TASK_004_SUMMARY.md](/workspaces/media-gateway/docs/BATCH_004_TASK_004_SUMMARY.md) - Implementation details
- [ab_testing_example.rs](/workspaces/media-gateway/examples/ab_testing_example.rs) - Usage example
- [ab_testing.rs](/workspaces/media-gateway/crates/sona/src/ab_testing.rs) - Source code
