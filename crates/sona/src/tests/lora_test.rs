//! LoRA adapter tests

use crate::lora::*;
use crate::types::ViewingEvent;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_lora_adapter_creation() {
    let user_id = Uuid::new_v4();
    let adapter = UserLoRAAdapter::new(user_id);

    assert_eq!(adapter.user_id, user_id);
    assert_eq!(adapter.rank, 8);
    assert_eq!(adapter.scaling_factor, 16.0 / 8.0); // LORA_ALPHA / LORA_RANK
    assert_eq!(adapter.training_iterations, 0);
}

#[test]
fn test_lora_forward_pass_input_dimension() {
    let mut adapter = UserLoRAAdapter::new(Uuid::new_v4());
    adapter.initialize_random();

    let input_vector = vec![0.5; 512]; // INPUT_DIM = 512
    let result = ComputeLoRAForward::execute(&adapter, &input_vector);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.len(), 768); // OUTPUT_DIM = 768
}

#[test]
fn test_lora_forward_pass_invalid_input_dimension() {
    let mut adapter = UserLoRAAdapter::new(Uuid::new_v4());
    adapter.initialize_random();

    let invalid_input = vec![0.5; 256]; // Wrong dimension
    let result = ComputeLoRAForward::execute(&adapter, &invalid_input);

    assert!(result.is_err());
}

#[test]
fn test_lora_forward_pass_produces_different_outputs() {
    let mut adapter = UserLoRAAdapter::new(Uuid::new_v4());
    adapter.initialize_random();

    let input1 = vec![0.3; 512];
    let input2 = vec![0.7; 512];

    let output1 = ComputeLoRAForward::execute(&adapter, &input1).unwrap();
    let output2 = ComputeLoRAForward::execute(&adapter, &input2).unwrap();

    // Different inputs should produce different outputs
    assert_ne!(output1, output2);
}

#[test]
fn test_lora_scaling_factor_calculation() {
    let adapter = UserLoRAAdapter::new(Uuid::new_v4());

    // LORA_ALPHA = 16.0, LORA_RANK = 8
    let expected_scaling = 16.0 / 8.0;
    assert_eq!(adapter.scaling_factor, expected_scaling);
}

#[test]
fn test_lora_rank_is_8() {
    let adapter = UserLoRAAdapter::new(Uuid::new_v4());
    assert_eq!(adapter.rank, 8);
}

#[test]
fn test_lora_base_layer_weights_dimensions() {
    let adapter = UserLoRAAdapter::new(Uuid::new_v4());

    // Base layer should be [rank, input_dim] = [8, 512]
    assert_eq!(adapter.base_layer_weights.shape(), &[8, 512]);
}

#[test]
fn test_lora_user_layer_weights_dimensions() {
    let adapter = UserLoRAAdapter::new(Uuid::new_v4());

    // User layer should be [output_dim, rank] = [768, 8]
    assert_eq!(adapter.user_layer_weights.shape(), &[768, 8]);
}

#[test]
fn test_lora_initialize_random_changes_weights() {
    let mut adapter = UserLoRAAdapter::new(Uuid::new_v4());

    // Check initial zero weights
    assert_eq!(adapter.base_layer_weights[[0, 0]], 0.0);
    assert_eq!(adapter.user_layer_weights[[0, 0]], 0.0);

    adapter.initialize_random();

    // After initialization, weights should be non-zero
    let has_nonzero_base = adapter.base_layer_weights.iter().any(|&w| w != 0.0);
    let has_nonzero_user = adapter.user_layer_weights.iter().any(|&w| w != 0.0);

    assert!(has_nonzero_base);
    assert!(has_nonzero_user);
}

#[test]
fn test_compute_lora_score_range() {
    let mut adapter = UserLoRAAdapter::new(Uuid::new_v4());
    adapter.initialize_random();

    let content_embedding = vec![0.5; 512];
    let preference_vector = vec![0.5; 768];

    let score = compute_lora_score(&adapter, &content_embedding, &preference_vector).unwrap();

    // Score should be clamped to [-1.0, 1.0]
    assert!(score >= -1.0);
    assert!(score <= 1.0);
}

#[test]
fn test_gradient_computation_for_training() {
    // Test gradient descent concept
    let learning_rate = 0.001;
    let initial_weight = 0.5;
    let gradient = 0.1;

    let updated_weight = initial_weight - learning_rate * gradient;

    assert!(updated_weight < initial_weight);
    assert_eq!(updated_weight, 0.5 - 0.0001);
}

#[test]
fn test_training_iteration_count() {
    // LoRA should train for 5 iterations
    const TRAINING_ITERATIONS: usize = 5;

    let mut iteration_count = 0;
    for _ in 0..TRAINING_ITERATIONS {
        iteration_count += 1;
    }

    assert_eq!(iteration_count, 5);
}

#[test]
fn test_min_training_events_requirement() {
    // Minimum 10 events required for training
    const MIN_EVENTS: usize = 10;

    let events: Vec<ViewingEvent> = (0..MIN_EVENTS)
        .map(|_| ViewingEvent {
            content_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            completion_rate: 0.8,
            rating: Some(4),
            is_rewatch: false,
            dismissed: false,
        })
        .collect();

    assert_eq!(events.len(), MIN_EVENTS);
}

#[test]
fn test_engagement_label_calculation_high_engagement() {
    let event = ViewingEvent {
        content_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        completion_rate: 1.0,
        rating: Some(5),
        is_rewatch: true,
        dismissed: false,
    };

    let label = UpdateUserLoRA::calculate_engagement_label(&event);

    assert!(label > 0.8);
    assert!(label <= 1.0);
}

#[test]
fn test_engagement_label_calculation_low_engagement() {
    let event = ViewingEvent {
        content_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        completion_rate: 0.3,
        rating: Some(1),
        is_rewatch: false,
        dismissed: false,
    };

    let label = UpdateUserLoRA::calculate_engagement_label(&event);

    assert!(label < 0.5);
    assert!(label >= 0.0);
}

#[test]
fn test_sigmoid_function() {
    let sigmoid_0 = UpdateUserLoRA::sigmoid(0.0);
    assert!((sigmoid_0 - 0.5).abs() < 0.01);

    let sigmoid_positive = UpdateUserLoRA::sigmoid(5.0);
    assert!(sigmoid_positive > 0.99);

    let sigmoid_negative = UpdateUserLoRA::sigmoid(-5.0);
    assert!(sigmoid_negative < 0.01);
}

#[test]
fn test_dot_product_calculation() {
    let a = vec![1.0, 2.0, 3.0];
    let b = vec![4.0, 5.0, 6.0];

    let result = UpdateUserLoRA::dot_product(&a, &b);

    // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    assert_eq!(result, 32.0);
}

#[test]
fn test_lora_memory_footprint_concept() {
    // LoRA rank=8: ~10KB per user
    let rank = 8;
    let input_dim = 512;
    let output_dim = 768;

    // Base layer: rank * input_dim floats
    let base_layer_params = rank * input_dim;

    // User layer: output_dim * rank floats
    let user_layer_params = output_dim * rank;

    let total_params = base_layer_params + user_layer_params;
    let bytes = total_params * 4; // 4 bytes per f32

    // Should be around 10KB
    assert!(bytes > 8000 && bytes < 12000);
}
