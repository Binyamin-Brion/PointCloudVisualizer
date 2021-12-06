#version 430 core

layout (location = 0) in vec2 aVertex;
layout (location = 1) in vec2 texCoords;

uniform float textScaleX;
uniform float textScaleY;

uniform mat4 projectionViewMatrix;
uniform vec2 translation;

out vec2 textureCoords;

void main()
{
    textureCoords = texCoords;

    vec4 positionVertex = vec4(aVertex, 0.0, 1.0);
    positionVertex.x *= 0.25 * textScaleX;
    positionVertex.y *= 0.25 * textScaleY;
    positionVertex.xy += translation;

    gl_Position = projectionViewMatrix * positionVertex;
}