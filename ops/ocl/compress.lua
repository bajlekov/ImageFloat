--[[
  Copyright (C) 2011-2021 G. Bajlekov

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
kernel void compress(global float *I, global float *H, global float *S, global float *O)
{
  const int x = get_global_id(0);
  const int y = get_global_id(1);

	float i = $I[x, y, 0];
	float h = $H[x, y, 0];
	float s = $S[x, y, 0];
	float o = $O[x, y, 0];

	float t = tanh(o*2.0f - 1.0f)*0.5f + 0.5f;
	float ch = o>0.5f ? t : o;
	float cs = o<0.5f ? t : o;

	float d = i-o;
	float vh = (1-h)*o + h*ch + d;
	float vs = (1-s)*o + s*cs + d;

	o = o>0.5f ? vh : vs;
	$O[x, y, 0] = o;
	$O[x, y, 1] = $I[x, y, 1] * o/i;
	$O[x, y, 2] = $I[x, y, 2] * o/i;
}
]]

local function execute()
	proc:getAllBuffers("I", "H", "S", "O")
	proc:executeKernel("compress", proc:size2D("O"))
end

local function init(d, c, q)
	proc:init(d, c, q)
	proc:loadSourceString(source)
	return execute
end

return init
