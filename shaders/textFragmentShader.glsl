#version 430 core

in vec2 textureCoords;

out vec4 FragColour;

layout (binding = 0) uniform sampler2D textBitmap;

void main()
{
   // FragColour = vec4(1.0, 0.0, 0.0, 1.0);
    FragColour = vec4(1.0, 1.0, 1.0, texture(textBitmap, textureCoords).r);
//    FragColour = vec4(texture(textBitmap, textureCoords).rgb, 1.0);
//FragColour = texture(textBitmap, textureCoords);
}