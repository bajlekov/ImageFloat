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

local proc = require "lib.opencl.process.ivy".new()

local source = [[
kernel curveRGB(I, C, O)
  const x = get_global_id(0)
  const y = get_global_id(1)
	const z = get_global_id(2)

  var j = YtoL(clamp(I[x, y, z], 0.0, 1.0))

  var lowIdx = clamp(int(floor(j*255)), 0, 255)
	var highIdx = clamp(int(ceil(j*255)), 0, 255)

	var lowVal = C[lowIdx, 0, z]
	var highVal = C[highIdx, 0, z]

  var factor = 0.0
  if lowIdx==highIdx then
    factor = 1.0
  else
    factor = j*255.0-lowIdx
  end

  O[x, y, z] = LtoY(mix(lowVal, highVal, factor))
end
]]

local function execute()
	local I, C, O = proc:getAllBuffers(3)
	proc:executeKernel("curveRGB", proc:size3D(O), {I, C, O})
end

local function init(d, c, q)
	proc:init(d, c, q)
	proc:loadSourceString(source)
	return execute
end

return init
