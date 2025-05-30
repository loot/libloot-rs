/*  LOOT

A load order optimisation tool for Oblivion, Skyrim, Fallout 3 and
Fallout: New Vegas.

Copyright (C) 2014-2016    WrinklyNinja

This file is part of LOOT.

LOOT is free software: you can redistribute
it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of
the License, or (at your option) any later version.

LOOT is distributed in the hope that it will
be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with LOOT.  If not, see
<https://www.gnu.org/licenses/>.
*/

#ifndef LOOT_TESTS_API_INTERFACE_METADATA_GROUP_TEST
#define LOOT_TESTS_API_INTERFACE_METADATA_GROUP_TEST

#include <gtest/gtest.h>

#include "loot/metadata/group.h"

namespace loot::test {
TEST(Group, defaultConstructorShouldCreateDefaultGroup) {
  Group group;

  EXPECT_EQ("default", group.GetName());
  EXPECT_TRUE(group.GetAfterGroups().empty());
}

TEST(Group,
     allArgsConstructorShouldSetDescriptionAndAfterGroupsDefaultsAsEmpty) {
  Group group("group1");

  EXPECT_EQ("group1", group.GetName());
  EXPECT_TRUE(group.GetDescription().empty());
  EXPECT_TRUE(group.GetAfterGroups().empty());
}

TEST(Group, allArgsConstructorShouldStoreGivenValues) {
  Group group("group1", {"other_group"}, "test");

  EXPECT_EQ("group1", group.GetName());
  EXPECT_EQ("test", group.GetDescription());
  EXPECT_EQ(std::vector<std::string>({"other_group"}), group.GetAfterGroups());
}

TEST(Group, equalityShouldBeCaseSensitiveOnNameAndDescription) {
  Group group1("name", {}, "description");
  Group group2("name", {}, "description");

  EXPECT_TRUE(group1 == group2);

  group1 = Group("name");
  group2 = Group("Name");

  EXPECT_FALSE(group1 == group2);

  group1 = Group("name", {}, "description");
  group2 = Group("name", {}, "Description");

  EXPECT_FALSE(group1 == group2);

  group1 = Group("name1");
  group2 = Group("name2");

  EXPECT_FALSE(group1 == group2);

  group1 = Group("name", {}, "description1");
  group2 = Group("name", {}, "description2");

  EXPECT_FALSE(group1 == group2);
}

TEST(Group, equalityShouldRequireEqualAfterGroups) {
  Group group1("name", {}, "description");
  Group group2("name", {}, "description");

  EXPECT_TRUE(group1 == group2);

  group1 = Group("name", {}, "description");
  group2 = Group("name", {"after1"}, "Description");

  EXPECT_FALSE(group1 == group2);
}

TEST(Group, inequalityShouldBeTheInverseOfEquality) {
  Group group1("name", {}, "description");
  Group group2("name", {}, "description");

  EXPECT_FALSE(group1 != group2);

  group1 = Group("name");
  group2 = Group("Name");

  EXPECT_TRUE(group1 != group2);

  group1 = Group("name", {}, "description");
  group2 = Group("name", {}, "Description");

  EXPECT_TRUE(group1 != group2);

  group1 = Group("name1");
  group2 = Group("name2");

  EXPECT_TRUE(group1 != group2);

  group1 = Group("name", {}, "description1");
  group2 = Group("name", {}, "description2");

  EXPECT_TRUE(group1 != group2);

  group1 = Group("name", {}, "description");
  group2 = Group("name", {"after1"}, "Description");

  EXPECT_TRUE(group1 != group2);
}

TEST(Group,
     lessThanOperatorShouldUseCaseSensitiveLexicographicalComparisonForNames) {
  Group group1("name", {}, "description");
  Group group2("name", {}, "description");

  EXPECT_FALSE(group1 < group2);
  EXPECT_FALSE(group2 < group1);

  group1 = Group("Name", {}, "description");
  group2 = Group("name", {}, "description");

  EXPECT_TRUE(group1 < group2);
  EXPECT_FALSE(group2 < group1);

  group1 = Group("name1", {}, "description");
  group2 = Group("name2", {}, "description");

  EXPECT_TRUE(group1 < group2);
  EXPECT_FALSE(group2 < group1);
}

TEST(
    Group,
    lessThanOperatorShouldUseCaseSensitiveLexicographicalComparisonForDescriptions) {
  Group group1("name", {}, "description");
  Group group2("name", {}, "description");

  EXPECT_FALSE(group1 < group2);
  EXPECT_FALSE(group2 < group1);

  group1 = Group("name", {}, "Description");
  group2 = Group("name", {}, "description");

  EXPECT_TRUE(group1 < group2);
  EXPECT_FALSE(group2 < group1);

  group1 = Group("name", {}, "description1");
  group2 = Group("name", {}, "description2");

  EXPECT_TRUE(group1 < group2);
  EXPECT_FALSE(group2 < group1);
}

TEST(Group, lessThanOperatorShouldCompareAfterGroups) {
  Group group1("name", {}, "description");
  Group group2("name", {}, "description");

  EXPECT_FALSE(group1 < group2);
  EXPECT_FALSE(group2 < group1);

  group1 = Group("name", {}, "description");
  group2 = Group("name", {"group"}, "description");

  EXPECT_TRUE(group1 < group2);
  EXPECT_FALSE(group2 < group1);

  group1 = Group("name", {"Group"}, "description");
  group2 = Group("name", {"group"}, "description");

  EXPECT_TRUE(group1 < group2);
  EXPECT_FALSE(group2 < group1);

  group1 = Group("name", {"group1"}, "description");
  group2 = Group("name", {"group2"}, "description");

  EXPECT_TRUE(group1 < group2);
  EXPECT_FALSE(group2 < group1);
}

TEST(Group,
     greaterThanOperatorShouldReturnTrueIfTheSecondGroupIsLessThanTheFirst) {
  Group group1("name", {}, "description");
  Group group2("name", {}, "description");

  EXPECT_FALSE(group1 > group2);
  EXPECT_FALSE(group2 > group1);

  group1 = Group("Name", {}, "description");
  group2 = Group("name", {}, "description");

  EXPECT_FALSE(group1 > group2);
  EXPECT_TRUE(group2 > group1);

  group1 = Group("name1", {}, "description");
  group2 = Group("name2", {}, "description");

  EXPECT_FALSE(group1 > group2);
  EXPECT_TRUE(group2 > group1);

  group1 = Group("name", {}, "Description");
  group2 = Group("name", {}, "description");

  EXPECT_FALSE(group1 > group2);
  EXPECT_TRUE(group2 > group1);

  group1 = Group("name", {}, "description1");
  group2 = Group("name", {}, "description2");

  EXPECT_FALSE(group1 > group2);
  EXPECT_TRUE(group2 > group1);

  group1 = Group("name", {}, "description");
  group2 = Group("name", {"group"}, "description");

  EXPECT_FALSE(group1 > group2);
  EXPECT_TRUE(group2 > group1);

  group1 = Group("name", {"Group"}, "description");
  group2 = Group("name", {"group"}, "description");

  EXPECT_FALSE(group1 > group2);
  EXPECT_TRUE(group2 > group1);

  group1 = Group("name", {"group1"}, "description");
  group2 = Group("name", {"group2"}, "description");

  EXPECT_FALSE(group1 > group2);
  EXPECT_TRUE(group2 > group1);
}

TEST(
    Group,
    lessThanOrEqualToOperatorShouldReturnTrueIfTheFirstGroupIsNotGreaterThanTheSecond) {
  Group group1("name", {}, "description");
  Group group2("name", {}, "description");

  EXPECT_TRUE(group1 <= group2);
  EXPECT_TRUE(group2 <= group1);

  group1 = Group("Name", {}, "description");
  group2 = Group("name", {}, "description");

  EXPECT_TRUE(group1 <= group2);
  EXPECT_FALSE(group2 <= group1);

  group1 = Group("name1", {}, "description");
  group2 = Group("name2", {}, "description");

  EXPECT_TRUE(group1 <= group2);
  EXPECT_FALSE(group2 <= group1);

  group1 = Group("name", {}, "Description");
  group2 = Group("name", {}, "description");

  EXPECT_TRUE(group1 <= group2);
  EXPECT_FALSE(group2 <= group1);

  group1 = Group("name", {}, "description1");
  group2 = Group("name", {}, "description2");

  EXPECT_TRUE(group1 <= group2);
  EXPECT_FALSE(group2 <= group1);

  group1 = Group("name", {}, "description");
  group2 = Group("name", {"group"}, "description");

  EXPECT_TRUE(group1 <= group2);
  EXPECT_FALSE(group2 <= group1);

  group1 = Group("name", {"Group"}, "description");
  group2 = Group("name", {"group"}, "description");

  EXPECT_TRUE(group1 <= group2);
  EXPECT_FALSE(group2 <= group1);

  group1 = Group("name", {"group1"}, "description");
  group2 = Group("name", {"group2"}, "description");

  EXPECT_TRUE(group1 <= group2);
  EXPECT_FALSE(group2 <= group1);
}

TEST(
    Group,
    greaterThanOrEqualToOperatorShouldReturnTrueIfTheFirstGroupIsNotLessThanTheSecond) {
  Group group1("name", {}, "description");
  Group group2("name", {}, "description");

  EXPECT_TRUE(group1 >= group2);
  EXPECT_TRUE(group2 >= group1);

  group1 = Group("Name", {}, "description");
  group2 = Group("name", {}, "description");

  EXPECT_FALSE(group1 >= group2);
  EXPECT_TRUE(group2 >= group1);

  group1 = Group("name1", {}, "description");
  group2 = Group("name2", {}, "description");

  EXPECT_FALSE(group1 >= group2);
  EXPECT_TRUE(group2 >= group1);

  group1 = Group("name", {}, "Description");
  group2 = Group("name", {}, "description");

  EXPECT_FALSE(group1 >= group2);
  EXPECT_TRUE(group2 >= group1);

  group1 = Group("name", {}, "description1");
  group2 = Group("name", {}, "description2");

  EXPECT_FALSE(group1 >= group2);
  EXPECT_TRUE(group2 >= group1);

  group1 = Group("name", {}, "description");
  group2 = Group("name", {"group"}, "description");

  EXPECT_FALSE(group1 >= group2);
  EXPECT_TRUE(group2 >= group1);

  group1 = Group("name", {"Group"}, "description");
  group2 = Group("name", {"group"}, "description");

  EXPECT_FALSE(group1 >= group2);
  EXPECT_TRUE(group2 >= group1);

  group1 = Group("name", {"group1"}, "description");
  group2 = Group("name", {"group2"}, "description");

  EXPECT_FALSE(group1 >= group2);
  EXPECT_TRUE(group2 >= group1);
}
}

#endif
