#version 430 core

// This is an uber-shader; required control flow is set through uniforms

in vec3 renderColour;
in flat uint sceneLightFragment;
in flat uint sideViewFragment;
in flat uint sideViewBorderFragment;
in vec2 textureCoords;
in vec4 lightSpaceVertex;
in vec3 normalizedVertexNormal;
in flat uint gridFragment;
in flat uint sceneFragment;
in flat uint drawingSideViewFragment;
in flat uint sunFragment;
in flat uint sunArrowFragment;
in vec3 fragPos;

out vec4 FragColour;

layout (binding = 0) uniform sampler2D rightViewTexture;

uniform vec3 borderColour;
uniform vec3 cameraPos;
uniform vec3 sunDirection;
uniform vec3 sunLightColour;

float pointInShadow()
{
    vec3 projectionCoords = (lightSpaceVertex.xyz / lightSpaceVertex.w) * 0.5 + 0.5;
    float lightViewDepth = texture(rightViewTexture, projectionCoords.xy).r;
    return (projectionCoords.z - 0.01) < lightViewDepth ? 1.0 : 0.50;
}

void main()
{
    if(sunArrowFragment == 1)
    {
        FragColour = vec4(0.5, 0.5, 0.0, 1.0);
    }
    else if(sunFragment == 1)
    {
        FragColour = vec4(0.6, 0.0, 0.0, 1.0);
    }
    else if(gridFragment == 1)
    {
        FragColour = vec4(renderColour, 1.0);
    }
    else if(sceneLightFragment == 1)
    {
        // Just record fragment depth; done by not writing anything
    }
    else if(sideViewBorderFragment == 1)
    {
         FragColour = vec4(borderColour, 1.0);
    }
    else if(sideViewFragment == 1)
    {
        float depthValue = texture(rightViewTexture, textureCoords).r;
        FragColour = vec4(depthValue);
    }
    else if(sideViewFragment == 2)
    {
        FragColour = texture(rightViewTexture, textureCoords);
    }
    else if(sceneFragment == 1)
    {
        vec3 ambientColour = vec3(0.4, 0.4, 0.4) * renderColour;

        float maxDotDiffuse = max(dot(-sunDirection, normalizedVertexNormal), 0.0);
        vec3 diffuseColour = maxDotDiffuse * sunLightColour * renderColour;

        vec3 cameraFragVector = normalize(cameraPos - fragPos);
        vec3 reflectDir = reflect(sunDirection, normalizedVertexNormal);
        vec3 halfwayVector = normalize(cameraFragVector + sunDirection);
        float maxDotSpecular = pow(max(dot(cameraFragVector, halfwayVector), 0.0), 32);
        vec3 specularColour = maxDotSpecular * sunLightColour * renderColour;

        vec3 finalColour = ambientColour + diffuseColour + specularColour;

        FragColour = vec4(finalColour * pointInShadow(), 1.0);
    }
    else if(drawingSideViewFragment == 1)
    {
        FragColour = vec4(renderColour, 1.0);
    }
}