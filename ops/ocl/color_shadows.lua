--[[
  Copyright (C) 2011-2019 G. Bajlekov

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
kernel void shadows(global float *I, global float *P, global float *O) {
	const int x = get_global_id(0);
	const int y = get_global_id(1);

	float3 i = $I[x, y];
	float3 p = $P[x, y];

	float v = $I[x, y]L;

	v = clamp(v, 0.0f, 1.0f);
	v = -2.0f*pown(v, 3) + 3.0f*pown(v, 2);
	v = 1-v;

	i = i + p*v;

	$O[x, y] = i;
}
]]

local function execute()
	proc:getAllBuffers("I", "P", "O")
	proc.buffers.I.__write = false
	proc.buffers.P.__write = false
	proc.buffers.O.__read = false
	proc:executeKernel("shadows", proc:size2D("O"))
end

local function init(d, c, q)
	proc:init(d, c, q)
	proc:loadSourceString(source)
	return execute
end

return init
