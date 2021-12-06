#include <fstream>
#include <iostream>
#include <open3d/geometry/PointCloud.h>
#include <cstdlib>

const unsigned int contentFileIndex = 1;
const unsigned int resultFileIndex = 2;
const unsigned int densityParameterIndex = 3;
const unsigned int minPointsClusterIndex = 4;

double stringToDouble(const std::string &number);
long stringToLong(const std::string &number);
std::vector<Eigen::Vector3d> extractPointsFromFile(std::ifstream &readStream);

int main(int argc, char *argv[])
{
    std::ifstream readStream{ argv[contentFileIndex], std::ifstream ::in };
    std::fstream resultStream{ argv[resultFileIndex], std::ofstream::out | std::ofstream::trunc };

    if (!readStream.is_open())
    {
        std::cerr << "Unable to open the read file: " << argv[contentFileIndex] << std::flush;
        exit(-1);
    }

    if (!resultStream.is_open())
    {
        std::cerr << "Unable to open the result file: " << argv[resultFileIndex] << std::flush;
        exit(-1);
    }

    double epos = stringToDouble(argv[densityParameterIndex]);
    size_t minPoints = stringToLong(argv[minPointsClusterIndex]);
    std::vector<Eigen::Vector3d> points = extractPointsFromFile(readStream);

    std::vector<int> clusterResult;

    try
    {
        open3d::geometry::PointCloud pointCloud{ points };
        clusterResult = pointCloud.ClusterDBSCAN(epos, minPoints, true);
    }
    catch(std::exception &e)
    {
        std::cerr << "Failed to find clusters on point cloud: " << e.what() << std::flush;
        exit(-1);
    }

    try
    {
        for(auto i : clusterResult)
        {
            resultStream << i << " " << std::flush;
        }
    }
    catch(std::exception &e)
    {
        std::cerr << "Failed to write point cloud result to file: " << e.what() << std::flush;
        exit(-1);
    }
}

double stringToDouble(const std::string &number)
{
    try
    {
        return std::strtod(number.c_str(), nullptr);
    }
    catch(std::out_of_range &e)
    {
        std::cerr << "Out of range string: " << number << "Error: " << e.what() << std::flush;
        exit(-1);
    }
    catch(std::invalid_argument &e)
    {
        std::cerr << "Invalid number string: " << number << "Error: " << e.what() << std::flush;
        exit(-1);
    }
}

long stringToLong(const std::string &number)
{
    try
    {
        return std::strtol(number.c_str(), nullptr, 10);
    }
    catch(std::out_of_range &e)
    {
        std::cerr << "Out of range string: " << number << "Error: " << e.what() << std::flush;
        exit(-1);
    }
    catch(std::invalid_argument &e)
    {
        std::cerr << "Invalid number string: " << number << "Error: " << e.what() << std::flush;
        exit(-1);
    }

}

std::vector<Eigen::Vector3d> extractPointsFromFile(std::ifstream &readStream)
{
    std::vector<Eigen::Vector3d> points;
    for (std::string contentLine; std::getline(readStream, contentLine);)
    {
        std::stringstream ss{ contentLine };
        std::string bufferString;
        unsigned int numberIndex = 1;
        double x, y, z;
        while (std::getline(ss, bufferString, '|'))
        {
            switch (numberIndex)
            {
                case 1:
                    x = stringToDouble(bufferString);
                    break;

                case 2:
                    y = stringToDouble(bufferString);
                    break;

                case 3:
                {
                    z = stringToDouble(bufferString);
                    points.emplace_back(x, y, z);
                    numberIndex = 0;
                }
                    break;

                default:
                    break;
            }

            numberIndex += 1;
        }
    }

    return points;
}