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

-- todo move imageSampleCoord to here! register previewImage

local function tweak()
	local o = {}

	local node

	local dx, dy =  0,  0
	local ox, oy = -1, -1
	local cx, cy = -1, -1

	local update = false

	local function imageSampleReleaseCallback()
		dx, dy = 0, 0
	end

	local function imageSampleDragCallback(mouse)
		node.dirty = true
		local shift = love.keyboard.isDown("lshift") or love.keyboard.isDown("rshift")
		dx = dx + (shift and mouse.dx/10 or mouse.dx)
		dy = dy + (shift and mouse.dy/10 or mouse.dy)
		update = true
		cx, cy = imageSample.coord(mouse.lx - mouse.ox + mouse.x, mouse.ly - mouse.oy + mouse.y)
		return imageSampleReleaseCallback
	end

	local function imageSamplePressCallback(frame, mouse)
		node.dirty = true
		update = true
    dx, dy = 0, 0
    ox, oy = imageSample.coord(mouse.lx, mouse.ly)
		cx, cy = ox, oy
    return imageSampleDragCallback
  end

	function o.getOrigin()
		return ox, oy
	end
	function o.getCurrent()
		local u = update
		update = false
		return cx, cy, u
	end
	function o.getUpdate()
		local u = update
		update = false
		return u
	end
	function o.getTweak()
		local x, y = dx, dy
		dx, dy = 0, 0
		return x, y
	end

	local function setToolCallback(elem)
		if elem.value then
			node = elem.parent
			imageSample.panel.onAction = imageSamplePressCallback
			dx, dy = 0, 0
		end
	end
	function o.toolButton(node, idx, name)
		local b = node:addElem("bool", idx, name, false)
    table.insert(imageSample.exclusive, b)
    for k, v in ipairs(imageSample.exclusive) do
      v.exclusive = imageSample.exclusive
    end
    b.onChange = setToolCallback
	end

	return o
end

return tweak
