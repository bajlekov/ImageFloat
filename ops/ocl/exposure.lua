--[[
  Copyright (C) 2011-2018 G. Bajlekov

    Ivy is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Ivy is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
]]

local proc = require "lib.opencl.process".new()

local source = [[
kernel void exposure(global float *p1, global float *p2, global float *p3)
{
  const int x = get_global_id(0);
  const int y = get_global_id(1);
	const int z = get_global_id(2);

  float i = $p1[x, y, z];
  float e = powr(2.0f, $p2[x, y, 0]);

  float o = i*e;

  $p3[x, y, z] = o;
}
]]

local function execute()
	proc:getAllBuffers("p1", "p2", "p3")
	proc:executeKernel("exposure", proc:size3D("p3"))
end

local function init(d, c, q)
	proc:init(d, c, q)
	proc:loadSourceString(source)
	return execute
end

return init
