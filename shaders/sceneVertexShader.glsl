#version 430 core

layout (location = 0) in vec3 vertex;
layout (location = 1) in vec2 texCoords;
layout (location = 2) in vec3 vertexNormal;
layout (location = 3) in vec3 pointColour;
layout (location = 4) in vec3 translation;

// This is an uber-shader; required control flow is set through uniforms

uniform int reflectVertically;
uniform vec3 cloudTranslation;
uniform uint renderSideViews;
uniform uint renderSideViewBorder;
uniform uint drawingSceneLightPerspective;
uniform uint drawingScene;
uniform uint drawingGrid;
uniform uint drawingFromSideView;
uniform uint drawingSun;
uniform uint drawingSunArrow;

uniform vec3 sunPosition;
uniform vec3 sunArrowPosition;
uniform float sunArrowScale;
uniform mat4 projViewMatrix;
uniform mat4 rotationMatrix;
uniform mat4 lightPerspectiveMatrix;

out flat uint sunFragment;
out flat uint sunArrowFragment;
out flat uint sideViewFragment;
out vec2 textureCoords;
out vec3 renderColour;
out flat uint sideViewBorderFragment;
out flat uint sceneLightFragment;
out vec4 lightSpaceVertex;
out vec3 normalizedVertexNormal;
out vec3 fragPos;
out flat uint gridFragment;
out flat uint sceneFragment;
out flat uint drawingSideViewFragment;

void main()
{
    renderColour = pointColour;
    sideViewFragment = renderSideViews;
    textureCoords = texCoords;
    sideViewBorderFragment = renderSideViewBorder;
    sceneLightFragment = drawingSceneLightPerspective;
    normalizedVertexNormal = vertexNormal;
    normalizedVertexNormal.y *= reflectVertically;
    normalizedVertexNormal = vec3(rotationMatrix * vec4(normalizedVertexNormal, 0.0)); // Normals are normalized during model loading
    gridFragment = drawingGrid;
    sceneFragment = drawingScene;
    drawingSideViewFragment = drawingFromSideView;
    sunFragment = drawingSun;
    sunArrowFragment = drawingSunArrow;

    if(drawingSunArrow == 1)
    {
         gl_Position = projViewMatrix * vec4(vertex * sunArrowScale + sunArrowPosition, 1.0);
    }
    if(drawingSun == 1)
    {
        gl_Position = projViewMatrix * vec4(vertex * 0.25 + sunPosition, 1.0);
    }
    else if(drawingGrid == 1)
    {
        gl_Position = projViewMatrix * vec4(vertex + translation, 1.0);
    }
    else if(renderSideViews == 1 || renderSideViews == 2 || renderSideViewBorder == 1)
    {
        gl_Position = rotationMatrix * vec4(vertex, 1.0);
    }
    else if(drawingSceneLightPerspective == 1)
    {
        vec4 worldSpaceVertex = vec4(vertex + translation + cloudTranslation, 1.0);
        worldSpaceVertex.y *= reflectVertically;
        gl_Position = projViewMatrix * worldSpaceVertex;
    }
    else if(drawingScene == 1)
    {
        vec4 worldSpaceVertex =  vec4(0.05 * vertex + translation + cloudTranslation + vec3(0.0, 0.995, 0.0), 1.0 );
        worldSpaceVertex.y *= reflectVertically;
        fragPos = worldSpaceVertex.xyz;
        gl_Position = projViewMatrix * worldSpaceVertex;
        lightSpaceVertex = lightPerspectiveMatrix * vec4(0.05 * vertex + translation + vec3(0.0, 0.995, 0.0), 1.0);
    }
    else if(drawingFromSideView == 1)
    {
        vec4 worldSpaceVertex = vec4(0.05 * vertex + translation + cloudTranslation + vec3(0.0, 0.995, 0.0), 1.0 );
        worldSpaceVertex.y *= reflectVertically;
        gl_Position = projViewMatrix * worldSpaceVertex;
    }
}