--[[
  Copyright (C) 2011-2018 G. Bajlekov

    ImageFloat is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    ImageFloat is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
]]

local proc = require "lib.opencl.process".new()

local source = [[
kernel void paste(global float *p1, global float *p2, global float *offset)
{
  const int x = get_global_id(0);
  const int y = get_global_id(1);
  const int z = get_global_id(2);

  float ox = offset[0];
  float oy = offset[1];
  float s = offset[2];

  //TODO: fix boundaries on write!!!
  $p2[(int)(x*s+ox), (int)(y*s+oy), z] = $p1[x, y, z];
}
]]

local function execute()
  proc:getAllBuffers("p1", "p2", "offset")
  proc:executeKernel("paste", proc:size3D("p1"))
end

local function init(d, c, q)
  proc:init(d, c, q)
  proc:loadSourceString(source)
  return execute
end

return init