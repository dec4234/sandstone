// Writes the hardness and the collision shape of every block state to a JSON file.
//
// The data generator reports do not have this data, so this program loads the vanilla classes and
// asks each block state. Run it with the unpacked server jar and its libraries on the class path:
//   java -cp <server jar>;<libraries> BlockPhysics <output file>
// The `report` command of the block_states tool compiles and runs it.

import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

public class BlockPhysics {
	public static void main(String[] args) throws Exception {
		if (args.length != 1) {
			System.err.println("usage: BlockPhysics <output file>");
			System.exit(2);
		}

		SharedConstants.tryDetectVersion();
		Bootstrap.bootStrap();

		// One line per state, in state ID order:
		// {"id": 1, "hardness": 1.5, "dynamic": false, "collision": [[minX, minY, minZ, maxX, maxY, maxZ], ...]}
		try (PrintWriter out = new PrintWriter(Files.newBufferedWriter(Path.of(args[0]), StandardCharsets.UTF_8))) {
			out.println("[");
			int count = Block.BLOCK_STATE_REGISTRY.size();
			for (int id = 0; id < count; id++) {
				BlockState state = Block.BLOCK_STATE_REGISTRY.byId(id);
				float hardness = state.getDestroySpeed(EmptyBlockGetter.INSTANCE, BlockPos.ZERO);

				// Some blocks (bamboo, pointed dripstone) move their shape by an offset that comes from
				// the position. Remove it, so the shape is the one at offset zero.
				Vec3 offset = state.getOffset(BlockPos.ZERO);
				// "dynamic" means the true shape can change with the position or the entity, so
				// the shape here is only a default.
				boolean dynamic = state.getBlock().hasDynamicShape() || state.hasOffsetFunction();

				StringBuilder boxes = new StringBuilder();
				for (AABB box : state.getCollisionShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO).toAabbs()) {
					if (boxes.length() > 0) {
						boxes.append(", ");
					}
					boxes.append('[')
						.append(box.minX - offset.x).append(", ")
						.append(box.minY - offset.y).append(", ")
						.append(box.minZ - offset.z).append(", ")
						.append(box.maxX - offset.x).append(", ")
						.append(box.maxY - offset.y).append(", ")
						.append(box.maxZ - offset.z).append(']');
				}

				out.printf("{\"id\": %d, \"hardness\": %s, \"dynamic\": %b, \"collision\": [%s]}%s%n",
					id, Float.toString(hardness), dynamic, boxes, id + 1 < count ? "," : "");
			}
			out.println("]");
		}
	}
}
