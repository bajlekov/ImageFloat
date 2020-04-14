--[[
  Copyright (C) 2011-2020 G. Bajlekov

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

local proc = require "lib.opencl.process.ivy".new()

local source = [[
kernel set_zero(O)
	const z = get_global_id(2)
	O[z] = 0.0
end

kernel sad(A, B, O)
	const y = get_global_id(1)
	const z = get_global_id(2)

	var s = 0.0
	for x = 0, max(A.x, B.x) - 1 do
		s = s + abs(A[x, y, z] - B[x, y, z])
	end

	atomic_add(O[z].ptr, s/(max(A.x, B.x) * max(A.y, B.y)))
end
]]

local function execute()
	local A, B, O = proc:getAllBuffers(3)
	local size = proc:size3Dmax(A, B)
	size[1] = 1

	proc:executeKernel("set_zero", proc:size3D(O), {O})
	proc:executeKernel("sad", size, {A, B, O})
end

local function init(d, c, q)
	proc:init(d, c, q)
	proc:loadSourceString(source)
	return execute
end

return init
