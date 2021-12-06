use nalgebra_glm::{TVec3, vec3};

/// Holds the colours that a cluster is to have
pub struct ClusterColour
{
    colours: Vec<TVec3<f32>>
}

impl ClusterColour
{
    /// Creates the colours that a cluster can have
    pub fn new() -> ClusterColour
    {
        let mut colours = Vec::new();

        let mut colour_intensity = 1.0;

        // Below are the easy to code colours. More color variations could be generated (such as by
        // having the RGB components be of different intensities, but this requires more code and
        // as of time writing, the number of colours generated below is sufficient

        while colour_intensity > 0.0
        {
            let adjusted_colour_intensity = colour_intensity * 0.7;

            colours.push(vec3(0.0, adjusted_colour_intensity, 0.0));
            colours.push(vec3(adjusted_colour_intensity, 0.0, 0.0));
            colours.push(vec3(0.0, 0.0, adjusted_colour_intensity));
            colours.push(vec3(adjusted_colour_intensity, adjusted_colour_intensity, 0.0));
            colours.push(vec3(adjusted_colour_intensity, 0.0, adjusted_colour_intensity));
            colours.push(vec3(0.0, adjusted_colour_intensity, adjusted_colour_intensity));
            colours.push(vec3(adjusted_colour_intensity, adjusted_colour_intensity, adjusted_colour_intensity));

            colour_intensity -= 0.1;
        }

        ClusterColour { colours }
    }

    /// Get the cluster colour given its index (as defined in the DBCluster scan). If the index
    /// is greater than the amount of colours prepared, then a non-unique colour is returned
    ///
    /// `cluster-index` - the index of the cluster according to the DBScan results
    pub fn get_colour(&self, cluster_index: usize) -> TVec3<f32>
    {
        if cluster_index >= self.colours.len()
        {
            vec3(1.0, 0.75, 0.5)
        }
        else
        {
            self.colours[cluster_index]
        }
    }
}