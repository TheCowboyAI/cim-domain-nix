# Conceptual Spaces Patterns

## Core Concepts

Conceptual spaces in CIM represent knowledge as geometric structures where:
- **Points** = Individual concepts/objects
- **Regions** = Categories/concepts
- **Dimensions** = Quality dimensions (features)
- **Distance** = Semantic similarity

## Implementation Patterns

### 1. Conceptual Space Definition

```rust
#[derive(Debug, Clone)]
pub struct ConceptualSpace {
    pub id: ConceptualSpaceId,
    pub dimensions: Vec<QualityDimension>,
    pub regions: HashMap<ConceptId, ConvexRegion>,
    pub embeddings: EmbeddingStore,
}

#[derive(Debug, Clone)]
pub struct QualityDimension {
    pub name: String,
    pub dimension_type: DimensionType,
    pub range: Range<f32>,
    pub metric: DistanceMetric,
}

#[derive(Debug, Clone)]
pub enum DimensionType {
    Continuous,      // e.g., temperature, size
    Categorical,     // e.g., color categories
    Ordinal,        // e.g., small < medium < large
    Circular,       // e.g., hue, direction
}
```

### 2. Embedding Integration

```rust
// Bridge between symbolic and geometric representations
pub struct EmbeddingBridge {
    pub model: EmbeddingModel,
    pub vector_store: VectorStore,
    pub dimension_mapping: DimensionMapping,
}

impl EmbeddingBridge {
    pub async fn embed_concept(
        &self,
        concept: &Concept,
    ) -> Result<ConceptualPoint> {
        // Generate embedding
        let embedding = self.model.embed(&concept.description).await?;

        // Map to quality dimensions
        let point = self.dimension_mapping.map_to_space(embedding)?;

        // Store for similarity search
        self.vector_store.insert(concept.id, point.clone()).await?;

        Ok(point)
    }
}
```

### 3. Convex Region Representation

```rust
// Natural categories form convex regions
#[derive(Debug, Clone)]
pub struct ConvexRegion {
    pub concept_id: ConceptId,
    pub prototype: ConceptualPoint,
    pub boundaries: Vec<Hyperplane>,
    pub member_points: HashSet<PointId>,
}

impl ConvexRegion {
    pub fn contains(&self, point: &ConceptualPoint) -> bool {
        // Point is inside if on correct side of all boundaries
        self.boundaries.iter().all(|plane| {
            plane.signed_distance(point) >= 0.0
        })
    }

    pub fn distance_to_prototype(&self, point: &ConceptualPoint) -> f32 {
        self.prototype.distance_to(point)
    }
}
```

### 4. Similarity and Distance Metrics

```rust
pub trait DistanceMetric {
    fn distance(&self, a: &ConceptualPoint, b: &ConceptualPoint) -> f32;
}

pub struct WeightedEuclidean {
    pub dimension_weights: Vec<f32>,
}

impl DistanceMetric for WeightedEuclidean {
    fn distance(&self, a: &ConceptualPoint, b: &ConceptualPoint) -> f32 {
        a.coordinates
            .iter()
            .zip(&b.coordinates)
            .zip(&self.dimension_weights)
            .map(|((a, b), w)| w * (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

// Context-sensitive similarity
pub struct ContextualSimilarity {
    pub base_metric: Box<dyn DistanceMetric>,
    pub context_weights: HashMap<ContextId, Vec<f32>>,
}
```

### 5. Knowledge Graph Integration

```rust
// Connect conceptual spaces with graph structure
pub struct ConceptualGraph {
    pub graph: StableGraph<ConceptNode, ConceptEdge>,
    pub spaces: HashMap<SpaceId, ConceptualSpace>,
    pub node_embeddings: HashMap<NodeId, ConceptualPoint>,
}

#[derive(Debug, Clone)]
pub struct ConceptNode {
    pub id: NodeId,
    pub concept: Concept,
    pub space_id: SpaceId,
    pub position: ConceptualPoint,
}

#[derive(Debug, Clone)]
pub struct ConceptEdge {
    pub relationship: SemanticRelation,
    pub strength: f32,
    pub derived_from_distance: bool,
}
```

## Event-Driven Conceptual Updates

### 1. Conceptual Space Events

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConceptualSpaceEvent {
    ConceptAdded {
        space_id: SpaceId,
        concept: Concept,
        position: ConceptualPoint,
    },
    RegionFormed {
        space_id: SpaceId,
        region: ConvexRegion,
        member_concepts: Vec<ConceptId>,
    },
    DimensionRecalibrated {
        space_id: SpaceId,
        dimension: QualityDimension,
        new_weights: Vec<f32>,
    },
    ConceptMoved {
        space_id: SpaceId,
        concept_id: ConceptId,
        old_position: ConceptualPoint,
        new_position: ConceptualPoint,
        reason: MoveReason,
    },
}
```

### 2. Learning and Adaptation

```rust
pub struct ConceptualLearning {
    pub learning_rate: f32,
    pub adaptation_threshold: f32,
}

impl ConceptualLearning {
    pub async fn adapt_from_feedback(
        &self,
        space: &mut ConceptualSpace,
        feedback: &UserFeedback,
    ) -> Result<Vec<ConceptualSpaceEvent>> {
        let mut events = Vec::new();

        match feedback {
            UserFeedback::SimilarityCorrection { a, b, should_be_similar } => {
                if *should_be_similar {
                    // Move concepts closer
                    let new_positions = self.attract_concepts(a, b, space)?;
                    events.push(ConceptualSpaceEvent::ConceptMoved { ... });
                } else {
                    // Move concepts apart
                    let new_positions = self.repel_concepts(a, b, space)?;
                    events.push(ConceptualSpaceEvent::ConceptMoved { ... });
                }
            }
            UserFeedback::CategoryCorrection { concept, category } => {
                // Adjust region boundaries
                let updated_region = self.adjust_region(concept, category, space)?;
                events.push(ConceptualSpaceEvent::RegionFormed { ... });
            }
        }

        Ok(events)
    }
}
```

## Integration with AI Agents

### 1. Semantic Navigation

```rust
pub struct SemanticNavigator {
    pub current_position: ConceptualPoint,
    pub goal_concept: ConceptId,
    pub path_constraints: Vec<PathConstraint>,
}

impl SemanticNavigator {
    pub fn plan_path(
        &self,
        space: &ConceptualSpace,
    ) -> Result<Vec<ConceptualPoint>> {
        // A* search through conceptual space
        // considering semantic distances
        let path = astar(
            &self.current_position,
            |p| self.neighbors(p, space),
            |p| self.heuristic(p, space),
            |p| self.is_goal(p),
        )?;

        Ok(path)
    }
}
```

### 2. Concept Synthesis

```rust
pub struct ConceptSynthesizer {
    pub combination_rules: Vec<CombinationRule>,
}

impl ConceptSynthesizer {
    pub fn synthesize_concept(
        &self,
        concepts: &[Concept],
        space: &ConceptualSpace,
    ) -> Result<Concept> {
        // Find centroid of input concepts
        let positions: Vec<_> = concepts.iter()
            .map(|c| space.get_position(c.id))
            .collect::<Result<_>>()?;

        let centroid = calculate_centroid(&positions);

        // Check if centroid falls in existing region
        if let Some(region) = space.find_region_containing(&centroid) {
            // Use prototype of that region
            Ok(region.prototype_concept())
        } else {
            // Create new concept at centroid
            let new_concept = Concept {
                id: ConceptId::new(),
                position: centroid,
                description: self.generate_description(&concepts),
                ..Default::default()
            };

            Ok(new_concept)
        }
    }
}
```

## Best Practices

### 1. Dimension Design

```rust
// Good: Orthogonal, interpretable dimensions
pub struct ColorSpace {
    pub hue: CircularDimension,        // 0-360 degrees
    pub saturation: LinearDimension,   // 0-1
    pub lightness: LinearDimension,    // 0-1
}

// Bad: Overlapping, unclear dimensions
pub struct BadColorSpace {
    pub redness: LinearDimension,      // Overlaps with others
    pub blueness: LinearDimension,     // Not orthogonal
    pub brightness: LinearDimension,   // Ambiguous
}
```

### 2. Region Formation

```rust
// Use prototype theory for natural categories
impl ConceptualSpace {
    pub fn form_region_from_examples(
        &mut self,
        examples: &[ConceptualPoint],
        negative_examples: &[ConceptualPoint],
    ) -> Result<ConvexRegion> {
        // Find prototype (centroid of examples)
        let prototype = calculate_centroid(examples);

        // Find boundaries using SVM or similar
        let boundaries = find_separating_hyperplanes(
            examples,
            negative_examples,
        )?;

        // Ensure convexity
        let convex_hull = ensure_convex_boundaries(boundaries)?;

        Ok(ConvexRegion {
            prototype,
            boundaries: convex_hull,
            member_points: examples.iter().map(|p| p.id).collect(),
        })
    }
}
```

### 3. Performance Optimization

```rust
// Use spatial indices for efficient queries
pub struct OptimizedConceptualSpace {
    pub space: ConceptualSpace,
    pub rtree: RTree<ConceptualPoint>,
    pub lsh: LocalitySensitiveHash,
}

impl OptimizedConceptualSpace {
    pub fn find_similar_concepts(
        &self,
        query: &ConceptualPoint,
        k: usize,
    ) -> Vec<(ConceptId, f32)> {
        // Use LSH for approximate nearest neighbors
        let candidates = self.lsh.query(query, k * 2);

        // Refine with exact distance
        let mut results: Vec<_> = candidates
            .into_iter()
            .map(|id| {
                let point = self.space.get_point(id).unwrap();
                let distance = self.space.distance(query, &point);
                (id, distance)
            })
            .collect();

        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);
        results
    }
}
```

## Testing Conceptual Spaces

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convexity() {
        let region = create_test_region();

        // Test that line between any two points stays in region
        for p1 in &region.member_points {
            for p2 in &region.member_points {
                let midpoint = interpolate(p1, p2, 0.5);
                assert!(region.contains(&midpoint));
            }
        }
    }

    #[test]
    fn test_prototype_theory() {
        let space = create_color_space();
        let red_examples = generate_red_examples();

        let red_region = space.form_region_from_examples(&red_examples).unwrap();

        // Prototype should be typical red
        assert_close_to(red_region.prototype.hue, 0.0, 0.1);
        assert!(red_region.prototype.saturation > 0.7);
    }
}
```

## Common Pitfalls

❌ **Too Many Dimensions**
- Keep dimensionality manageable (typically < 100)
- Use dimensionality reduction if needed

❌ **Non-Convex Regions**
- Natural categories should be convex
- Use multiple regions for complex concepts

❌ **Ignoring Context**
- Similarity is context-dependent
- Allow for multiple spaces/views

✅ **Best Practices**
- Design interpretable dimensions
- Use prototype theory for categories
- Maintain convexity of regions
- Index for performance
- Allow for adaptation and learning